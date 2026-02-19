CREATE TABLE IF NOT EXISTS data_api_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    request_id TEXT,
    schema_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    action TEXT NOT NULL,
    principal TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    status_code INTEGER NOT NULL,
    details TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_data_api_audit_logs_occurred_at
    ON data_api_audit_logs (occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_data_api_audit_logs_tenant_table
    ON data_api_audit_logs (tenant_id, table_name);

CREATE INDEX IF NOT EXISTS idx_data_api_audit_logs_request_id
    ON data_api_audit_logs (request_id);

CREATE TABLE IF NOT EXISTS data_api_table_metadata (
    tenant_id UUID NOT NULL,
    schema_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    exposed BOOLEAN NOT NULL DEFAULT TRUE,
    read_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    create_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    update_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    delete_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    introspect_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    authorization_mode TEXT NOT NULL DEFAULT 'authenticated',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, schema_name, table_name),
    CONSTRAINT fk_data_api_table_metadata_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES provisioned_databases (id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS data_api_column_metadata (
    tenant_id UUID NOT NULL,
    schema_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    column_name TEXT NOT NULL,
    readable BOOLEAN NOT NULL DEFAULT TRUE,
    writable BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, schema_name, table_name, column_name),
    CONSTRAINT fk_data_api_column_metadata_table
        FOREIGN KEY (tenant_id, schema_name, table_name)
        REFERENCES data_api_table_metadata (tenant_id, schema_name, table_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_data_api_table_metadata_tenant
    ON data_api_table_metadata (tenant_id);

CREATE INDEX IF NOT EXISTS idx_data_api_column_metadata_tenant_table
    ON data_api_column_metadata (tenant_id, schema_name, table_name);
