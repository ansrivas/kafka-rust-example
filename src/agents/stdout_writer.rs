use crate::agents::agent::{Agent, Payload};
use crate::errors::AppError;
use async_trait::async_trait;
use log::info;
use tokio::time::Duration;

#[derive(Clone)]
pub struct StdOutWriter {
	pub topic: String,
	pub print_seconds: Duration,
}

impl StdOutWriter {
	/// Create a new instance of StdOutWriter
	pub fn new<T: Into<String>>(print_seconds: Duration, topic: T) -> Self {
		Self {
			topic: topic.into(),
			print_seconds: print_seconds,
		}
	}

	async fn echo(&self, payload: &str) -> Result<(), AppError> {
		info!("Writing to stdout. {}", payload);
		Ok(())
	}
}

#[async_trait]
impl Agent for StdOutWriter {
	fn validate(&self, _raw_data: &[u8]) -> Result<Payload, AppError> {
		Ok(Payload::CustomString("some string".into()))
	}

	async fn run(&self, raw_data: &[u8]) -> Result<(), AppError> {
		let payload = self.validate(raw_data)?;
		let msg = match payload {
			Payload::CustomString(msg) => msg,
			_ => {
				return Err(AppError::CustomErr(
					"[StdOutWriter] Unsupported message received".into(),
				))
			}
		};
		self.echo(&msg).await?;
		Ok(())
	}

	fn topic(&self) -> &str {
		self.topic.as_ref()
	}

	fn name(&self) -> &str {
		"StdOutWriter"
	}

	fn consumer_group(&self) -> &str {
		"StdOutWriter"
	}
}
