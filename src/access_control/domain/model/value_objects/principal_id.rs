use std::fmt;
use uuid::Uuid;
use crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PrincipalId(Uuid);

impl PrincipalId {
    pub fn new(value: String) -> Result<Self, AccessControlDomainError> {
        let trimmed = value.trim();
        let uuid = Uuid::parse_str(trimmed)
            .map_err(|_| AccessControlDomainError::InvalidPrincipalId)?;
        Ok(Self(uuid))
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for PrincipalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
