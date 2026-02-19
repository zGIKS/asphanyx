CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS provisioned_databases (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    database_name VARCHAR(63) PRIMARY KEY,
    username VARCHAR(63) NOT NULL,
    password_hash TEXT NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'provisioned_databases_id_unique'
    ) THEN
        ALTER TABLE provisioned_databases
            ADD CONSTRAINT provisioned_databases_id_unique UNIQUE (id);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_provisioned_databases_id
    ON provisioned_databases (id);

CREATE TABLE IF NOT EXISTS provisioning_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
