use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::provisioner::domain::model::enums::provisioner_domain_error::ProvisionerDomainError;

#[derive(Clone, Debug)]
pub struct ProvisioningAuditEventRecord {
    event_name: String,
    database_name: String,
    username: Option<String>,
    status: String,
    error_message: Option<String>,
    occurred_at: DateTime<Utc>,
}

impl ProvisioningAuditEventRecord {
    pub fn new(
        event_name: impl Into<String>,
        database_name: impl Into<String>,
        username: Option<String>,
        status: impl Into<String>,
        error_message: Option<String>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            event_name: event_name.into(),
            database_name: database_name.into(),
            username,
            status: status.into(),
            error_message,
            occurred_at,
        }
    }

    pub fn event_name(&self) -> &str {
        &self.event_name
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[async_trait]
pub trait ProvisioningAuditEventRepository: Send + Sync {
    async fn save_event(
        &self,
        event: &ProvisioningAuditEventRecord,
    ) -> Result<(), ProvisionerDomainError>;
}
