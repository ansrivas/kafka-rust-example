-- Add migration script here
CREATE INDEX ON metrics (name, timestamp DESC);