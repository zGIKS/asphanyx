CREATE TABLE IF NOT EXISTS data_api_audit_logs (
    id BIGSERIAL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
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
