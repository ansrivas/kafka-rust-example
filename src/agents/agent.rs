use crate::errors::AppError;
use crate::generated::BatchMessage;
use async_trait::async_trait;

pub enum Payload {
	BatchMessage(BatchMessage),
}

#[async_trait]
pub trait Agent {
	fn validate(&self, raw_data: &[u8]) -> Result<Payload, AppError>;
	async fn run(&self, raw_data: &[u8]) -> Result<(), AppError>;
}
