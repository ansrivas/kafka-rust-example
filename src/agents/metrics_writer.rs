use crate::agents::agent::{Agent, Payload};
use crate::errors::AppError;
use crate::generated::BatchMessage;
use crate::postgres::DbClient;
use async_trait::async_trait;
use log::{debug, error, info};
use prost::Message as PMessage;

pub struct MetricsWriter {
	pub dbclient: DbClient,
	pub topic: String,
}

impl MetricsWriter {
	/// Create a new instance of MetricsWriter
	pub fn new<T: Into<String>>(dbclient: DbClient, topic: T) -> Self {
		Self {
			dbclient,
			topic: topic.into(),
		}
	}

	async fn metrics_writer(&self, payload: &BatchMessage) -> Result<(), AppError> {
		info!("Waiting to receive metrics-data on incoming queue.");
		debug!("Received data on the incoming channel to write in database");
		if let Err(e) = self.dbclient.insert(payload).await {
			error!("Failed to write data to the db: {:?}", e);
			let _ = self.dbclient.insert(payload).await;
		}
		Ok(())
	}
}

#[async_trait]
impl Agent for MetricsWriter {
	fn validate(&self, raw_data: &[u8]) -> Result<Payload, AppError> {
		let payload = BatchMessage::decode(raw_data)?;
		Ok(Payload::BatchMessage(payload))
	}

	async fn run(&self, raw_data: &[u8]) -> Result<(), AppError> {
		let payload = self.validate(raw_data)?;
		let batch_message = match payload {
			Payload::BatchMessage(batch_message) => batch_message,
			_ => return Err(AppError::CustomErr("Unsupported message received".into())),
		};
		if self.topic == "ankur" {
			log::info!("Should have written");
		} else {
			self.metrics_writer(&batch_message).await?;
		}
		Ok(())
	}

	fn topic(&self) -> String {
		self.topic.clone()
	}
}
