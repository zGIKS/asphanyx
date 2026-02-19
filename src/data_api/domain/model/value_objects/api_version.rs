use crate::data_api::domain::model::enums::data_api_domain_error::DataApiDomainError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiVersion(String);

impl ApiVersion {
    pub fn new(value: String) -> Result<Self, DataApiDomainError> {
        if value != "v1" {
            return Err(DataApiDomainError::UnsupportedApiVersion);
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
