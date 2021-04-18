// MIT License
//
// Copyright (c) 2019 Ankur Srivastava
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

pub mod config;
mod errors;
pub mod generated;
pub mod kafka;
pub mod metrics;
pub mod postgres;

use prost::bytes::BytesMut;

use config::Config;
use uuid::Uuid;

use generated::BatchMessage;
use kafka::{KafkaConsumer, KafkaProducer};
use log::{debug, error, info};
use metrics::MetricsGenerator;
use postgres::DbClient;
use prost::Message as PMessage;
use rdkafka::{
	config::{ClientConfig, RDKafkaLogLevel},
	consumer::stream_consumer::StreamConsumer,
};
use std::sync::Arc;
use structopt::{clap::Shell, StructOpt};
use tokio::{self, sync::mpsc, task, time, time::Duration};

#[derive(Debug, StructOpt)]
pub enum Command {
	#[structopt(name = "metrics-publisher")]
	/// Start publishing the metrics data to a kafka-topic.
	MetricsPublisher,

	#[structopt(name = "check-db-data")]
	/// Check the current count of rows in database.
	CheckDbData,

	#[structopt(name = "metrics-subscriber")]
	/// Subscribe to a kafka-topic and write data to database.
	MetricsSubscriber,
}

#[derive(Debug, StructOpt)]
pub struct Metrics {
	#[structopt(subcommand)]
	pub command: Command,
}

/// Create a consumer based on the given configuration.
///
/// In case certificate path etc is provided then a sasl enabled client
/// is created else a normal client.
fn create_consumer(conf: Arc<Config>) -> KafkaConsumer {
	let is_tls = conf.kafka_ca_cert_path.is_some()
		&& conf.kafka_password.is_some()
		&& conf.kafka_username.is_some();

	if is_tls {
		info!("TLS is enabled. Will try to create a secure client");
		let username = conf
			.kafka_username
			.as_deref()
			.expect("Kafka username is required.");
		let password = conf
			.kafka_password
			.as_deref()
			.expect("Kafka password is required.");
		let ca_path = conf
			.kafka_ca_cert_path
			.as_deref()
			.expect("Kafka ca certificate is required.");
		let consumer: StreamConsumer = ClientConfig::new()
			.set("group.id", "some-random-id")
			.set("bootstrap.servers", &conf.kafka_brokers)
			.set("enable.partition.eof", "false")
			.set("session.timeout.ms", "6000")
			.set("enable.auto.commit", "true")
			.set("sasl.mechanisms", "PLAIN")
			.set("security.protocol", "SASL_SSL")
			.set("sasl.username", username)
			.set("sasl.password", password)
			.set("ssl.ca.location", ca_path)
			.set_log_level(RDKafkaLogLevel::Debug)
			.create()
			.expect("Consumer creation failed");
		return KafkaConsumer::new_with_consumer(consumer, &[&conf.kafka_topic]);
	}

	let group_id = Uuid::new_v4();
	KafkaConsumer::new(
		&conf.kafka_brokers,
		&group_id.to_string(),
		&[&conf.kafka_topic],
	)
}

/// Create a producer based on the given configuration.
///
/// In case certificate path etc is provided then a sasl enabled client
/// is created else a normal client.
fn create_producer(conf: Arc<Config>) -> KafkaProducer {
	let is_tls = conf.kafka_ca_cert_path.is_some()
		&& conf.kafka_password.is_some()
		&& conf.kafka_username.is_some();

	if is_tls {
		let username = conf
			.kafka_username
			.as_deref()
			.expect("Kafka username is required.");
		let password = conf
			.kafka_password
			.as_deref()
			.expect("Kafka password is required.");
		let ca_path = conf
			.kafka_ca_cert_path
			.as_deref()
			.expect("Kafka ca certificate is required.");
		let producer = ClientConfig::new()
			.set("bootstrap.servers", &conf.kafka_brokers)
			.set("message.timeout.ms", "10000")
			.set("sasl.mechanisms", "PLAIN")
			.set("security.protocol", "SASL_SSL")
			.set("sasl.username", username)
			.set("sasl.password", password)
			.set("ssl.ca.location", ca_path)
			.create()
			.expect("Producer creation error");
		return KafkaProducer::new_with_producer(producer);
	}

	KafkaProducer::new(&conf.kafka_brokers)
}

/// Handle the message subscription command.
///
/// This will subscribe to a kafka-topic on which metrics are being published.
/// Then the incoming message is deserialized back to BatchMessage and
/// published to an internal channel.
/// Then this data is read and published to postgres.
async fn handle_message_receiving(config: Arc<Config>, dbclient: DbClient) {
	let (dbtx, mut dbrx) = mpsc::channel(100);
	task::spawn(async move {
		info!("Waiting to receive metrics-data on incoming queue.");
		while let Some(raw_data) = dbrx.recv().await {
			debug!("Received data on the incoming channel to write in database");
			if let Ok(bmsg) = BatchMessage::decode(raw_data) {
				if let Err(e) = dbclient.insert(&bmsg).await {
					error!("Failed to write data to the db: {:?}", e);
					let _ = dbclient.insert(&bmsg).await;
				}
			} else {
				error!("Failed to decode the incoming message from kafka");
			};
		}
	});

	debug!("Starting to cosume the data");
	let conf = config.clone();
	let kconsumer = create_consumer(conf);
	kconsumer.consume(dbtx).await;
}

/// Handle the message publishing command.
///
/// This will generate metrics, convert it to protobuf messages of type
/// BatchMessage and covert it to bytes
/// Send this message to an internal channel which is then consumed
/// by a kafka producer to publish this message to a kafka-topic.
async fn handle_message_publishing(config: Arc<Config>) {
	// Create a mpsc channel to publish data to
	let (tx, mut rx) = mpsc::channel(100);
	let mut batch_messages = BatchMessage::default();

	// Spawn an async task to collect metrics
	task::spawn(async move {
		debug!("Starting to produce the data");

		let mut interval = time::interval(Duration::from_millis(1000));
		loop {
			interval.tick().await;
			// This is in its own scope so that it gets collected and
			// ulimits are respected
			{
				let metrics_generator = MetricsGenerator::new();
				let mut metrices = metrics_generator.used_memory();
				let disks = metrics_generator.disk_stats();
				// ...
				// ... simulate some more statistics here and extend them all in metrics vector
				metrices.extend(disks);
				batch_messages.multiple_points = metrices;
			}
			let mut buffer = BytesMut::with_capacity(batch_messages.encoded_len());
			batch_messages.encode(&mut buffer).unwrap();

			if let Err(e) = tx.send(buffer).await {
				error!("receiver dropped {e}", e = e);
				return;
			};
		}
	});

	let conf = config.clone();

	// Create a kafka producer
	let kproducer = create_producer(conf);

	// Start reading data in the main thread
	// and publish it to Kafka
	while let Some(data) = rx.recv().await {
		debug!("Received data on the incoming channel");
		kproducer.produce(data, &config.kafka_topic).await;
		info!(
			"Published data successfully on kafka topic: {}",
			&config.kafka_topic
		);
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	Metrics::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
	let opt = Metrics::from_args();
	debug!("starting up");

	let app_config = Arc::new(Config::new());

	let dbclient = DbClient::from(
		&app_config.postgres_database_url,
		app_config.postgres_cert_path.as_deref(),
	)?;

	match opt.command {
		Command::MetricsPublisher => {
			info!("Started metrics publishing to kafka-topic");
			handle_message_publishing(app_config.clone()).await
		}
		Command::MetricsSubscriber => {
			info!("Subscriber was invoked");
			handle_message_receiving(app_config.clone(), dbclient).await
		}
		Command::CheckDbData => {
			let rows = dbclient.get_count().await?;
			info!("Current count of rows in DB is {:?}", rows);
		}
	};
	Ok(())
}
