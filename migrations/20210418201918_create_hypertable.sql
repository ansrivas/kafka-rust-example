-- Add migration script here
SELECT create_hypertable('metrics', 'timestamp');