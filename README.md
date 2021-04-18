# kafka-rust-example

This is an application which

- reads bunch of metrics from your machine/docker-container
- publishes it to Kafka topic `metrics`
- subscribes to Kafka topic `metrics`
- reads the information from that topic and writes to postgres `defaultdb.metrics` on remote server

### Why this application?

Simply to make myself aware of the followings in rust-ecosystem:

- Protobuf in rust
- Kafka in rust
- Postgres in rust

### Design:

- The application is divided into two sub-parts: `metrics-publisher` and `metrics-subscriber`. Each of these two subparts is exposed as a command line entry point.

- `metrics-publisher`:

  - Launches a async-task to collect metrices to publish data on an tokio::sync::mpsc channel.
  - Another async-task listens to this channel and publishes this data to Kafka topic `metrics`
  - The messages are protobuf encoded and are sent out in batches. More details in `data/message.proto`
  - To edit the protobuf-message format, edit the `data/message.proto` file and re-generate the definitions using:

  ```
  make generate-proto
  ```

- `metrics-subscriber`:
  - Launches a async-task to listen to a Kafka topic `metrics`.
  - Each incoming protobuf-message is deserialized and published on the internal tokio::sync::mpsc channel
  - On receiving messages the database async-task writes this to the database.

### For database migrations
```
cargo install sqlx-cli
make migrations
```
### Configuration:

There are two methods for the execution - dockerized and local installation. You need to follow until step 5 for both the methods.
The default configuration is in `config/env.dev`.

### Local Development Mode:

- For local development please run `docker-compose up -d`
- This will launch a bunch of docker containers locally, check it by running `docker ps`

  ```
  $ docker ps
  docker ps
  CONTAINER ID        IMAGE                               COMMAND                  PORTS                      NAMES
  00886360bf34        timescale/timescaledb:latest-pg11   "docker-entrypoint.s…"   0.0.0.0:5432->5432/tcp     timescale-db
  0c78b8ad9c6c        postgres:11-alpine                  "docker-entrypoint.s…"   0.0.0.0:54321->5432/tcp    pg-docker
  6022042c83fe        confluentinc/cp-kafka:5.3.1         "/etc/confluent/dock…"   0.0.0.0:9092->9092/tcp     kafka

  ```

- Run the migration using ( ensure you have `cargo install sqlx-cli`)
  ```
   make migrations
  ```
- After building the project (`cargo build`), you will find a binary inside `target/debug/kafka-rust-example`
- Run the publisher as:
  ```
  RUST_LOG=info APPLICATION_CONFIG_PATH=./config/env.dev ./target/debug/kafka-rust-example metrics-publisher
  ```
- Run the subscriber as:
  ```
  RUST_LOG=debug APPLICATION_CONFIG_PATH=./config/env.dev ./target/debug/kafka-rust-example metrics-subscriber
  ```
- Check rows in DB:
  ```
  RUST_LOG=info APPLICATION_CONFIG_PATH=./config/env.dev ./target/debug/kafka-rust-example check-db-data
  ```

## License

MIT
