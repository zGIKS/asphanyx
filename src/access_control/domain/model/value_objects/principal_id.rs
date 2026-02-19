use crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PrincipalId(String);

impl PrincipalId {
    pub fn new(value: String) -> Result<Self, AccessControlDomainError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AccessControlDomainError::InvalidPrincipalId);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
