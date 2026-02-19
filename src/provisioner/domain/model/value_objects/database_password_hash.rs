use crate::provisioner::domain::model::enums::provisioner_domain_error::ProvisionerDomainError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatabasePasswordHash(String);

impl DatabasePasswordHash {
    pub fn new(value: String) -> Result<Self, ProvisionerDomainError> {
        let trimmed = value.trim().to_string();

        if trimmed.len() < 16 {
            return Err(ProvisionerDomainError::InvalidDatabasePassword);
        }

        Ok(Self(trimmed))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
