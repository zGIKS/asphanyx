use async_trait::async_trait;

use crate::data_api::domain::model::{
    enums::data_api_domain_error::DataApiDomainError,
    events::data_api_request_audited_event::DataApiRequestAuditedEvent,
};

#[async_trait]
pub trait DataApiAuditLogRepository: Send + Sync {
    async fn save_event(
        &self,
        event: &DataApiRequestAuditedEvent,
    ) -> Result<(), DataApiDomainError>;
}
