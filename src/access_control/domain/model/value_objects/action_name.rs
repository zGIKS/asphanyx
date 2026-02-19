use crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ActionName(String);

impl ActionName {
    pub fn new(value: String) -> Result<Self, AccessControlDomainError> {
        match value.as_str() {
            "read" | "create" | "update" | "delete" | "*" => Ok(Self(value)),
            _ => Err(AccessControlDomainError::InvalidActionName),
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
