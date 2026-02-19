use crate::data_api::domain::model::enums::data_api_domain_error::DataApiDomainError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ColumnName(String);

impl ColumnName {
    pub fn new(value: String) -> Result<Self, DataApiDomainError> {
        let valid = !value.trim().is_empty()
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

        if !valid {
            return Err(DataApiDomainError::InvalidColumnName);
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
