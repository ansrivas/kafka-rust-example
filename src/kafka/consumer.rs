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

use futures::StreamExt;
use log::{debug, error, warn};

use prost::bytes::BytesMut;

use rdkafka::{
	config::{ClientConfig, RDKafkaLogLevel},
	consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
	message::Message,
};
use tokio::{self, sync::mpsc};

pub struct KafkaConsumer {
	kafka_consumer: StreamConsumer,
}

impl KafkaConsumer {
	/// Create a new KafkaConsumer.
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let consumer = KafkaConsumer::new("localhost:9092", "my-unique-group", &["topic1"]);
	/// ```
	pub fn new(kafka_brokers: &str, group_id: &str, topics: &[&str]) -> KafkaConsumer {
		// Create the `Futureconsumer` to produce asynchronously.
		let consumer: StreamConsumer = ClientConfig::new()
			.set("group.id", group_id)
			.set("bootstrap.servers", kafka_brokers)
			.set("enable.partition.eof", "false")
			.set("session.timeout.ms", "6000")
			.set("enable.auto.commit", "true")
			.set_log_level(RDKafkaLogLevel::Debug)
			.create()
			.expect("Consumer creation failed");

		consumer
			.subscribe(topics)
			.expect("Failed to subscribe to specified topics");

		KafkaConsumer {
			kafka_consumer: consumer,
		}
	}

	pub fn new_with_consumer(consumer: StreamConsumer, topics: &[&str]) -> KafkaConsumer {
		consumer
			.subscribe(topics)
			.expect("Failed to subscribe to specified topics");

		KafkaConsumer {
			kafka_consumer: consumer,
		}
	}

	/// Consume the incoming topic and publishes the raw-payload to an internal
	/// mpsc channel to be consumed by another async-task which then writes the
	/// data to postgres.
	pub async fn consume(&self, sender_tx: mpsc::Sender<BytesMut>) {
		debug!("initiating data consumption from kafka-topic");

		let mut message_stream = self.kafka_consumer.stream();
		while let Some(message) = message_stream.next().await {
			match message {
				Err(e) => warn!("Kafka error: {}", e),
				Ok(m) => {
					if let Some(raw_data) = m.payload() {
						debug!(
							"Received message on Kafka {:?} on offset {:?}",
							&raw_data,
							m.offset()
						);
						let payload = BytesMut::from(raw_data);
						if let Err(e) = &sender_tx.send(payload).await {
							error!("receiver dropped: {:?}", e);
						}
					} else {
						warn!("Failed to read raw data from kafka topic")
					}

					if let Err(e) = self.kafka_consumer.commit_message(&m, CommitMode::Async) {
						error!("Failed to commit offset to kafka: {:?}", e);
					}
				}
			};
		}
		debug!("Returned from consumer");
	}
}
