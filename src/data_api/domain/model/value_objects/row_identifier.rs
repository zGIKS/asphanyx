use crate::data_api::domain::model::enums::data_api_domain_error::DataApiDomainError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowIdentifier(String);

impl RowIdentifier {
    pub fn new(value: String) -> Result<Self, DataApiDomainError> {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.len() > 255 {
            return Err(DataApiDomainError::InvalidRowIdentifier);
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
