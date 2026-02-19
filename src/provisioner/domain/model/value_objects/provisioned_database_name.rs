use regex::Regex;

use crate::provisioner::domain::model::enums::provisioner_domain_error::ProvisionerDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProvisionedDatabaseName(String);

impl ProvisionedDatabaseName {
    pub fn new(value: String) -> Result<Self, ProvisionerDomainError> {
        let normalized = value.trim().to_lowercase();
        let regex = Regex::new(r"^[a-z][a-z0-9_]{2,62}$").expect("valid regex");

        if !regex.is_match(&normalized) {
            return Err(ProvisionerDomainError::InvalidDatabaseName);
        }

        Ok(Self(normalized))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
