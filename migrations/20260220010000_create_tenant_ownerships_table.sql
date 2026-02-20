CREATE TABLE IF NOT EXISTS tenant_ownerships (
    tenant_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_tenant_ownerships_tenant_id
        FOREIGN KEY (tenant_id)
        REFERENCES provisioned_databases(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tenant_ownerships_user_id
    ON tenant_ownerships (user_id);

CREATE INDEX IF NOT EXISTS idx_tenant_ownerships_tenant_id
    ON tenant_ownerships (tenant_id);
