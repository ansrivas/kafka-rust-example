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

use crate::{
	errors::AppError,
	generated::{BatchMessage, Message},
};
use chrono::prelude::*;
use deadpool_postgres::{Manager, Pool};
use log::info;
use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use std::fs;
use tokio_postgres::Config;

pub struct DbClient {
	pool: Pool,
}

impl DbClient {
	/// Create a DbClient from username, password, etc.
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let conn_string = "postgresql://postgres:password@localhost:5432/timeseries";
	/// let client = DbClient::new("localhost", "5432", "username", "password", "dbname");
	/// // use this client from this point on.
	/// ```
	pub fn new(host: &str, port: &str, username: &str, password: &str, dbname: &str) -> DbClient {
		let mut cfg = Config::new();
		cfg.host(host);
		cfg.port(port.parse::<u16>().unwrap());
		cfg.user(username);
		cfg.password(password);
		cfg.dbname(dbname);
		let mgr = Manager::new(cfg, tokio_postgres::NoTls);
		let connection_pool = Pool::builder(mgr)
			.max_size(16)
			.build()
			.expect("Failed to create a pool");
		DbClient {
			pool: connection_pool,
		}
	}

	/// Create a DBClient from a connection-string and certificate path.
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let conn_string = "postgresql://postgres:password@localhost:5432/timeseries";
	/// let client = DBClient::from(conn_string, path_to_cert);
	/// // use this client from this point on.
	/// ```
	pub fn from(conn_string: &str, cert_path: Option<&str>) -> Result<DbClient, AppError> {
		let config = conn_string
			.parse::<Config>()
			.expect("Failed to parse db-connection string");

		let pool = if let Some(cert_path) = cert_path {
			let connector = DbClient::create_tls_connection(cert_path)?;
			let mgr = Manager::new(config, connector);
			Pool::builder(mgr).max_size(16).build()?
		} else {
			let mgr = Manager::new(config, tokio_postgres::NoTls);
			Pool::builder(mgr).max_size(16).build()?
		};

		Ok(DbClient { pool })
	}

	fn create_tls_connection(path: &str) -> Result<MakeTlsConnector, AppError> {
		let cert_path = fs::read(path)?;
		// .unwrap_or_else(|_| panic!("Failed to read the cert file from path: {}", path));

		let cert = Certificate::from_pem(&cert_path)?;
		// .unwrap_or_else(|_| panic!("Failed to create the certificate from path: {}", path));

		let connector = TlsConnector::builder()
			.add_root_certificate(cert)
			.build()
			.expect("Failed to create a tls connector for Postgres");

		Ok(MakeTlsConnector::new(connector))
	}

	/// Get current count of rows in the database.
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let conn_string = "postgresql://postgres:password@localhost:5432/timeseries";
	/// let client = DBClient::from(conn_string, path_to_cert);
	/// client.get_count().await.unwrap();
	/// // use this client from this point on.
	/// ```
	pub async fn get_count(&self) -> Result<i64, AppError> {
		let client = self.pool.get().await?;
		let stmt = client.prepare("SELECT COUNT(*) FROM metrics").await?;
		let rows = client.query(&stmt, &[]).await?;
		let value: i64 = rows[0].get(0);
		Ok(value)
	}

	/// Insert a batch message in database
	///
	/// # Examples
	/// Basic usage:
	///
	/// ```rust norun
	/// let conn_string = "postgresql://postgres:password@localhost:5432/timeseries";
	/// let client = DBClient::from(conn_string, path_to_cert);
	/// let batch_message = BatchMessage::default();
	/// client.get_count().insert(batch_message).unwrap();
	/// // use this client from this point on.
	/// ```
	pub async fn insert(&self, messages: &BatchMessage) -> Result<(), AppError> {
		let client = self.pool.get().await?;
		let stmt = client
			.prepare("INSERT INTO metrics (timestamp, name, value) VALUES ($1, $2, $3)")
			.await?;

		for message in messages.multiple_points.iter() {
			let ts = DateTime::<Utc>::from_utc(
				NaiveDateTime::from_timestamp_opt(message.timestamp, 0).unwrap(),
				Utc,
			);
			client
				.execute(&stmt, &[&ts, &message.name, &(message.value as f64)])
				.await?;
		}
		info!("Published data to db");
		Ok(())
	}

	/// Insert a single message in database
	///
	/// # Examples
	///
	/// ```rust norun
	/// let client = DBClient::new("localhost", "5432", "username", "password", "metrics");
	/// let some_message = Message::Default();
	/// client.insert_message(some_message).await.unwrap();
	/// ```
	pub async fn insert_message(&self, message: &Message) -> Result<(), AppError> {
		let client = self.pool.get().await?;
		let stmt = client
			.prepare("INSERT INTO metrics (timestamp, name, value) VALUES ($1, $2, $3)")
			.await?;

		let ts = DateTime::<Utc>::from_utc(
			NaiveDateTime::from_timestamp_opt(message.timestamp, 0).unwrap(),
			Utc,
		);
		client
			.execute(&stmt, &[&ts, &message.name, &(message.value as f64)])
			.await?;
		info!("Published data to db");
		Ok(())
	}

	/// Truncate the table which contains all the metrics.
	///
	/// # Examples
	///
	/// ```rust norun
	/// let client = DBClient::new("localhost", "5432", "username", "password", "metrics");
	/// // Clean up the DB first
	/// client.truncate().await.unwrap();
	/// ```
	#[allow(dead_code)]
	pub(crate) async fn truncate(&self) -> Result<(), AppError> {
		let client = self.pool.get().await?;
		let stmt = client.prepare("TRUNCATE TABLE metrics").await?;
		client.execute(&stmt, &[]).await?;

		info!("Published data to db");
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::MetricsGenerator;
	#[tokio::test]
	async fn test_insert_single_message() {
		let client = DbClient::new("localhost", "5432", "postgres", "password", "timeseries");

		// Clean up the DB first
		client.truncate().await.unwrap();
		// Now insert a new row
		let message = MetricsGenerator::create_metrics("user".to_string(), 321f32, None);
		client.insert_message(&message).await.unwrap();

		// Now get the count of rows
		let expected = 1;
		let actual = client.get_count().await.unwrap();
		assert!(
			actual == expected,
			"Failed tests expected: {:?}, actual: {:?}",
			expected,
			actual
		);
	}

	#[tokio::test]
	async fn test_insert_batch_message() {
		let client = DbClient::new("localhost", "5432", "postgres", "password", "timeseries");

		// Clean up the DB first
		client.truncate().await.unwrap();

		// Now insert a new row
		let message1 = MetricsGenerator::create_metrics("user1".to_string(), 321f32, None);
		let message2 = MetricsGenerator::create_metrics("user2".to_string(), 321f32, None);

		let mut batch_message = BatchMessage::default();
		batch_message.multiple_points = vec![message1, message2];
		client.insert(&batch_message).await.unwrap();

		// Now get the count of rows
		let expected = 2;
		let actual = client.get_count().await.unwrap();
		assert!(
			actual == expected,
			"Failed tests expected: {:?}, actual: {:?}",
			expected,
			actual
		);
	}
}
