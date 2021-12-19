use crate::errors::AppError;
use crate::generated::BatchMessage;
use async_trait::async_trait;

pub enum Payload {
	BatchMessage(BatchMessage),
	CustomString(String),
}

pub trait CloneableAgent {
	fn clone_box(&self) -> Box<dyn Agent>;
}

impl<T> CloneableAgent for T
where
	T: 'static + Agent + Clone,
{
	fn clone_box(&self) -> Box<dyn Agent> {
		Box::new(self.clone())
	}
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Agent> {
	fn clone(&self) -> Box<dyn Agent> {
		self.clone_box()
	}
}

#[async_trait]
pub trait Agent: Sync + Send + CloneableAgent {
	/// Validate the incoming raw_data
	/// The data from Kafka is going to be byte array, you can validate
	/// your payload here and take some action
	fn validate(&self, raw_data: &[u8]) -> Result<Payload, AppError>;

	/// Override the run method according to your implementation
	/// for e.g. if the agent needs to write the raw data to DB
	/// this is where it can do that
	async fn run(&self, raw_data: &[u8]) -> Result<(), AppError>;

	/// Topic from which this agent is going to consume the data
	fn topic(&self) -> &str;

	/// Representative name of this agent
	fn name(&self) -> &str;

	/// Default concurrency for the agent
	/// This means that the app will spawn 3 consumers
	/// for this agent.
	fn concurrency(&self) -> i32 {
		3
	}

	/// Unique consumer group for this Agent
	fn consumer_group(&self) -> &str;
}
