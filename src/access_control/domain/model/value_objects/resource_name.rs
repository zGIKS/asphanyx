use crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ResourceName(String);

impl ResourceName {
    pub fn new(value: String) -> Result<Self, AccessControlDomainError> {
        if value == "*" {
            return Ok(Self(value));
        }

        let valid = !value.trim().is_empty()
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

        if !valid {
            return Err(AccessControlDomainError::InvalidResourceName);
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
