# aiven-rs

This is an application which
- reads bunch of metrics from your machine/docker-container
- publishes it to Aiven-Kafka topic `metrics`
- subscribes to Aiven-Kafka topic `metrics`
- reads the information from that topic and writes to Aiven-postgres `defaultdb.metrics` on remote server

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


### Installation
Use a virtual environment with `python 3.7` installed in it:
```
pip install -e .[dev]
```

### Connection to Aiven Platform:
There are two methods for the execution - dockerized and local installation. You need to follow until step 5 for both the methods.

1. Login with your credentials on `https://console.aiven.io/`
2. Go to `Create a new service`
3. Postgres:
  - Launch a postgres-v11 service on a cloud of your choice
  - Copy the service-uri and put this in `alembic.ini` file:
    - **IMPORTANT NOTE**: Change the `postgres://` to `postgresql://` in your service-uri.
    ```
    sqlalchemy.url=<service-uri-postgresql>
    ```
  - Also download the `CA-Certificate` from `Overview` tab and put it in ``certs/ca-postgres.pem``
  - Now edit the configuration file `config/env.prod`
    - `APPLICATION_POSTGRES_DATABASE_URL=<service-uri-postgresql>`
    - `APPLICATION_POSTGRES_CERT_PATH=<path-to-ca-certificate>`
  - At this point run `make migrations` to create the tables etc.

4. Kafka:
  - Launch a kafka-cluster- v2.4
  - When the cluster is ready, download the CA-certificate from `Overview` tab and put it in `certs/ca-kafka.pem`
  - While on `Overview` tab, scroll down to `Advanced Configuration` and
    - enable `kafka_authentication_methods.sasl`
    - enable `kafka.auto_create_topics_enable` ( You can manually create the topic `metrics` as well.)
    - and hit `Save Advanced Configuration`.
  - After saving the configuration you will notice `Authentication method` under the `Overview` tab, set it to `SASL`
  - Edit again the `config/env.prod` file:
    - APPLICATION_KAFKA_BROKERS="<Kafka-Service-URI"
    - APPLICATION_KAFKA_TOPIC="metrics"
    - APPLICATION_KAFKA_USERNAME="Username-from-users-tab"
    - APPLICATION_KAFKA_PASSWORD="Password-from-users-tab"
    - APPLICATION_KAFKA_CA_CERT_PATH="Path-to-downloaded Kafka CA certificate"

5. Now we are ready with the configurations.

### Local Installation Method
1. At this point inside a virtual-environment simply run `pip install -e .[dev]`
2. After installation of the project from **Installation** stage we will get these three binaries in our path:
`metrics-publisher`,  `metrics-subscriber` and `check-db-data`.
3. Use the configuration file locally `config/env.prod`
4. In one terminal, launch the subscriber:
```
$ RUST_LOG=info APPLICATION_CONFIG_PATH=./config/env.prod metrics-subscriber --loglevel=DEBUG
```
5. In another terminal launch the publisher:
```
$ RUST_LOG=info APPLICATION_CONFIG_PATH=./config/env.prod metrics-publisher --loglevel=DEBUG
```
6. Once everything works perfectly fine, simply run this to check the amount of data being written currently
```
RUST_LOG=info APPLICATION_CONFIG_PATH=./config/env.prod check-db-data
```

### Dockerized Method:
1. Dockerized method can be used after `step 5`.
2. This step assumes that your config file is `config/env.prod`
3. `alembic.ini` is populated properly
4. Certificates are in `certs` directory.
5. Thus, you build the docker container first:
  ```
  make build_docker
  ```
6. Run the migrations:
  ```
  make dockerized_migrations
  ```
7. In one terminal spawn the publisher:
  ```
  make dockerized_publisher
  ```

8. In another spawn the consumer:
  ```
  make dockerized_subscriber
  ```

9. After everything runs, simply count the rows in the remote db:
  ```
  make dockerized_check_db_data
  ```


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
- Run the migration using ( given `alembic.ini` is pointing to localhost-postgres)
  ```
   make migrations
  ```
- After building the project, you will find a binary inside `target/debug/aiven-rs`
- Run the publisher as:
  ```
  RUST_LOG=info APPLICATION_CONFIG_PATH=./config/env.dev ./target/debug/aiven-rs metrics-publisher
  ```
- Run the subscriber as:
  ```
  RUST_LOG=debug APPLICATION_CONFIG_PATH=./config/env.dev ./target/debug/aiven-rs metrics-subscriber
  ```
- Check rows in DB:
  ```
  RUST_LOG=info APPLICATION_CONFIG_PATH=./config/env.dev ./target/debug/aiven-rs check-db-data
  ```
## License
MIT
