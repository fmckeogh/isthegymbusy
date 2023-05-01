-- Add migration script here
CREATE TABLE IF NOT EXISTS measurements (
    measured_at TIMESTAMPTZ NOT NULL PRIMARY KEY DEFAULT CURRENT_TIMESTAMP,
    value SMALLINT NOT NULL CHECK (value >= 0)
);
