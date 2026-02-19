use std::fmt;
use uuid::Uuid;
use crate::data_api::domain::model::enums::data_api_domain_error::DataApiDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(Uuid);

impl TenantId {
    pub fn new(value: String) -> Result<Self, DataApiDomainError> {
        let trimmed = value.trim();
        let uuid = Uuid::parse_str(trimmed)
            .map_err(|_| DataApiDomainError::InvalidTenantId)?;
        Ok(Self(uuid))
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
