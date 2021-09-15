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

use crate::generated::Message;
use chrono::prelude::*;
use sysinfo::{DiskExt, System, SystemExt};

#[derive(Default)]
pub struct MetricsGenerator {
	pub client: System,
}

impl MetricsGenerator {
	/// Create a new instance of MetricsGenerator.
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let mg = MetricsGenerator::new();
	/// ```
	pub fn new() -> Self {
		MetricsGenerator {
			client: System::new(),
		}
	}

	/// Create a metrics message out of given entries.
	/// In case timestamp is not provided Utc::now() is set as timestamp entry.
	pub(crate) fn create_metrics(name: String, value: f32, timestamp: Option<i64>) -> Message {
		Message {
			timestamp: timestamp.unwrap_or_else(|| Utc::now().timestamp_millis()),
			name,
			value,
		}
	}

	/// Generate disk stats from running operating system.
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let mg = MetricsGenerator::new();
	/// let metrics = mg.disk_stats();
	/// ```
	pub fn disk_stats(&self) -> Vec<Message> {
		let mut messages = vec![];
		for (idx, disk) in self.client.disks().iter().enumerate() {
			let _metrics_name = format!("disk-available-space-{idx}", idx = idx);
			let metrics = Self::create_metrics(
				disk.name().to_os_string().into_string().unwrap(),
				disk.available_space() as f32,
				None,
			);
			messages.push(metrics);
		}
		messages
	}

	/// Generate used memory from running operating system.
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let mg = MetricsGenerator::new();
	/// let metrics = mg.used_memory();
	/// ```
	pub fn used_memory(&self) -> Vec<Message> {
		let mut messages = vec![];
		let message = Self::create_metrics(
			"used-memory".to_string(),
			self.client.used_memory() as f32,
			None,
		);
		messages.push(message);
		messages
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_used_memory() {
		let mg = MetricsGenerator::new();
		let messages = mg.used_memory();
		assert!(
			messages.len() > 0,
			"Should be able to collect the used memory"
		);

		let message = messages[0].clone();
		assert!(message.name == "used-memory".to_string());
		assert!(
			message.value >= 0.0f32,
			"Actual value was {:?}",
			message.value
		);
	}
}
