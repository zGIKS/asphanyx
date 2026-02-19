CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS provisioned_databases (
    database_name VARCHAR(63) PRIMARY KEY,
    username VARCHAR(63) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS provisioning_audit_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    event_name VARCHAR(64) NOT NULL,
    database_name VARCHAR(63) NOT NULL,
    username VARCHAR(63),
    status VARCHAR(20) NOT NULL,
    error_message TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_provisioning_audit_events_database_name
    ON provisioning_audit_events (database_name);

CREATE INDEX IF NOT EXISTS idx_provisioning_audit_events_occurred_at
    ON provisioning_audit_events (occurred_at DESC);
