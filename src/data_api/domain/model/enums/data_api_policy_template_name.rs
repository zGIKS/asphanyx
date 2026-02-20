use crate::data_api::domain::model::enums::data_api_domain_error::DataApiDomainError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataApiPolicyTemplateName {
    AclCrud,
    AclReadOnly,
    AuthenticatedCrud,
}

impl DataApiPolicyTemplateName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AclCrud => "acl_crud",
            Self::AclReadOnly => "acl_read_only",
            Self::AuthenticatedCrud => "authenticated_crud",
        }
    }

    pub fn authorization_mode(&self) -> &'static str {
        match self {
            Self::AclCrud | Self::AclReadOnly => "acl",
            Self::AuthenticatedCrud => "authenticated",
        }
    }

    pub fn metadata_flags(&self) -> (bool, bool, bool, bool, bool, bool) {
        match self {
            Self::AclCrud | Self::AuthenticatedCrud => (true, true, true, true, true, true),
            Self::AclReadOnly => (true, true, false, false, false, true),
        }
    }

    pub fn parse(value: &str) -> Result<Self, DataApiDomainError> {
        match value.trim() {
            "acl_crud" => Ok(Self::AclCrud),
            "acl_read_only" => Ok(Self::AclReadOnly),
            "authenticated_crud" => Ok(Self::AuthenticatedCrud),
            _ => Err(DataApiDomainError::InvalidPolicyTemplateName),
        }
    }

    pub fn all() -> &'static [Self] {
        const VALUES: [DataApiPolicyTemplateName; 3] = [
            DataApiPolicyTemplateName::AclCrud,
            DataApiPolicyTemplateName::AclReadOnly,
            DataApiPolicyTemplateName::AuthenticatedCrud,
        ];
        &VALUES
    }
}
