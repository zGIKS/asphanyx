use uuid::Uuid;

use crate::provisioner::domain::model::enums::provisioner_domain_error::ProvisionerDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProvisionedDatabaseId(Uuid);

impl ProvisionedDatabaseId {
    pub fn new(value: String) -> Result<Self, ProvisionerDomainError> {
        let parsed = Uuid::parse_str(value.trim()).map_err(|_| {
            ProvisionerDomainError::InfrastructureError("invalid database id".to_string())
        })?;
        Ok(Self(parsed))
    }

    pub fn new_random() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}
