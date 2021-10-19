#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchMessage {
	#[prost(message, repeated, tag = "3")]
	pub multiple_points: ::prost::alloc::vec::Vec<Message>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
	#[prost(int64, tag = "1")]
	pub timestamp: i64,
	#[prost(string, tag = "2")]
	pub name: ::prost::alloc::string::String,
	#[prost(float, tag = "3")]
	pub value: f32,
}
