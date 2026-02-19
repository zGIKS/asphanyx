CREATE TABLE IF NOT EXISTS access_role_assignments (
    tenant_id UUID NOT NULL,
    principal_id UUID NOT NULL,
    role_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, principal_id, role_name)
);

CREATE TABLE IF NOT EXISTS access_policy_rules (
    tenant_id UUID NOT NULL,
    role_name TEXT NOT NULL,
    resource_name TEXT NOT NULL,
    action_name TEXT NOT NULL,
    effect TEXT NOT NULL,
    allowed_columns TEXT[],
    owner_scope BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, role_name, resource_name, action_name)
);

CREATE TABLE IF NOT EXISTS access_authorization_decision_audit (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL,
    principal_id UUID NOT NULL,
    resource_name TEXT NOT NULL,
    action_name TEXT NOT NULL,
    allowed BOOLEAN NOT NULL,
    reason TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_access_role_assignments_tenant_principal
    ON access_role_assignments (tenant_id, principal_id);

CREATE INDEX IF NOT EXISTS idx_access_policy_rules_lookup
    ON access_policy_rules (tenant_id, resource_name, action_name, role_name);

CREATE INDEX IF NOT EXISTS idx_access_authorization_decision_audit_occurred_at
    ON access_authorization_decision_audit (occurred_at DESC);
