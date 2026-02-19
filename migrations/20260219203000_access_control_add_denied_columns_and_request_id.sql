ALTER TABLE access_policy_rules
    ADD COLUMN IF NOT EXISTS denied_columns TEXT[];

ALTER TABLE access_authorization_decision_audit
    ADD COLUMN IF NOT EXISTS request_id TEXT;

CREATE INDEX IF NOT EXISTS idx_access_authorization_decision_audit_request_id
    ON access_authorization_decision_audit (request_id);
