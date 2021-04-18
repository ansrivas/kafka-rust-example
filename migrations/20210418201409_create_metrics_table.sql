-- Add migration script here

CREATE TABLE metrics (
    timestamp TIMESTAMPTZ PRIMARY KEY,
    name TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL
);