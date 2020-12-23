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

use std::env;

use log::info;
use serde::Deserialize;
const DEFAULT_CONFIG_ENV_KEY: &str = "APPLICATION_CONFIG_PATH";
const CONFIG_PREFIX: &str = "APPLICATION_";

struct ConfigFn {}

#[allow(dead_code)]
impl ConfigFn {
	fn fn_false() -> bool {
		false
	}
	fn fn_true() -> bool {
		true
	}
	fn fn_default_host() -> String {
		"0.0.0.0".into()
	}
	fn fn_default_port() -> String {
		"8080".into()
	}
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
	/// Run the app in debug mode
	#[serde(default = "ConfigFn::fn_true")]
	pub debug: bool,

	/// Set the address to bind the webserver on
	/// defaults to 0.0.0.0:8080
	#[serde(default = "ConfigFn::fn_default_host")]
	pub host: String,

	/// Default port, soon deprecated.
	#[serde(default = "ConfigFn::fn_default_port")]
	pub port: String,

	/// Kafka topic on which we want to publish the data.
	pub kafka_topic: String,

	/// Kafka brokers to connect to.
	pub kafka_brokers: String,

	/// Kafka username for sasl authentication.
	pub kafka_username: Option<String>,

	/// Kafka password for sasl authentication.
	pub kafka_password: Option<String>,

	/// Kafka ca-cert path for sasl authentication.
	pub kafka_ca_cert_path: Option<String>,

	/// Postgres database url
	pub postgres_database_url: String,

	/// Postgres path to cert.
	pub postgres_cert_path: Option<String>,
}

impl Config {
	// Create a new Config instance by reading from
	// environment variables
	pub fn new() -> Config {
		// Check if there is an environment variable DEVICE_REGISTRY_CONFIG_PATH
		// then read it from there else fallback to .env
		let filename = match env::var(DEFAULT_CONFIG_ENV_KEY) {
			Ok(filepath) => filepath,
			Err(_) => ".env".into(),
		};
		info!("Trying to read the config file from [{}]", &filename);

		dotenv::from_filename(&filename).ok();
		match envy::prefixed(CONFIG_PREFIX).from_env::<Config>() {
			Ok(config) => config,
			Err(e) => panic!("Config file being read: {}. And error {:?}", &filename, e),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn eq_with_nan_eq(a: &Config, b: &Config) -> bool {
		return (a.host == b.host) && (a.port == b.port) && (a.debug == b.debug);
	}

	fn vec_compare(va: &[Config], vb: &[Config]) -> bool {
		// zip stops at the shortest
		(va.len() == vb.len()) && va.iter().zip(vb).all(|(a, b)| eq_with_nan_eq(a, b))
	}

	#[test]
	fn test_config_parsing() {
		let json = r#"
	[
	  {
		"debug": false,
        "postgres_database_url": "localhost",
        "kafka_brokers": "localhost:9092",
        "kafka_topic": "metrics"
	  },
	  {
		  "host": "127.0.0.1",
          "port": "9080",
          "postgres_database_url": "localhost",
          "kafka_brokers": "localhost:9092",
          "kafka_topic": "metrics"
	  }
	]
"#;
		let config: Vec<Config> = serde_json::from_str(json).unwrap();

		let expected_config: Vec<Config> = vec![
			Config {
				debug: false,
				host: "0.0.0.0".into(),
				port: "8080".into(),
				postgres_database_url: "localhost".into(),
				kafka_brokers: "localhost:9092".into(),
				kafka_topic: "metrics".into(),
				postgres_cert_path: None,
				kafka_ca_cert_path: None,
				kafka_username: None,
				kafka_password: None,
			},
			Config {
				debug: true,
				host: "127.0.0.1".into(),
				port: "9080".into(),
				postgres_database_url: "localhost".into(),
				kafka_brokers: "localhost:9092".into(),
				kafka_topic: "metrics".into(),
				postgres_cert_path: None,
				kafka_ca_cert_path: None,
				kafka_username: None,
				kafka_password: None,
			},
		];
		assert_eq!(
			vec_compare(&config, &expected_config),
			true,
			"Parsing failed !!!"
		);
	}

	#[test]
	fn test_config_reading() {
		let path = env::var("CARGO_MANIFEST_DIR");
		let env_file = format!("{}/config/env.dev", path.unwrap());
		env::set_var(DEFAULT_CONFIG_ENV_KEY, env_file);
		let config: Config = Config::new();
		assert!(config.kafka_brokers == "localhost:9092");
		assert!(config.kafka_topic == "metrics");
		assert!(config.debug);
	}
}
