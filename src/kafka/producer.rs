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

use log::{debug, error};
use prost::bytes::BytesMut;
use rdkafka::{
	config::ClientConfig,
	producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;

pub struct KafkaProducer {
	producer: FutureProducer,
}

impl KafkaProducer {
	/// Create a new KafkaProducer instance with a provided FutureProducer
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let kproducer = KafkaProducer::new("localhost:9092");
	/// ```
	pub fn new(kafka_brokers: &str) -> KafkaProducer {
		// Create the `FutureProducer` to produce asynchronously.
		let kafka_producer: FutureProducer = ClientConfig::new()
			.set("bootstrap.servers", kafka_brokers)
			.set("message.timeout.ms", "10000")
			.create()
			.expect("Producer creation error");
		KafkaProducer {
			producer: kafka_producer,
		}
	}

	/// Create a new KafkaProducer instance with a provided FutureProducer
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let producer = ClientConfig::new()
	///     .set("bootstrap.servers", &conf.kafka_brokers)
	///     .set("message.timeout.ms", "10000")
	///     .create()
	///     .expect("Producer creation error");
	/// let kproducer = KafkaProducer::new_with_producer(producer);
	/// ```
	pub fn new_with_producer(kafka_producer: FutureProducer) -> KafkaProducer {
		KafkaProducer {
			producer: kafka_producer,
		}
	}

	/// Publish a BytesMut record to a given topic on Kafka.
	pub async fn produce(&self, data: BytesMut, topic: &str) {
		let record = FutureRecord::to(topic).key("some key").payload(&data[..]);
		// let produce_future: DeliveryFuture = self.producer.send(record, 0);
		let produce_future = self.producer.send(record, Duration::from_millis(100)).await;
		match produce_future {
			Ok(message) => debug!("Status: {:?}", message),
			Err(_) => error!("Future cancelled"),
		};
	}
}
