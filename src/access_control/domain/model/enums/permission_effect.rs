use std::str::FromStr;

use super::access_control_domain_error::AccessControlDomainError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionEffect {
    Allow,
    Deny,
}

impl PermissionEffect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

impl FromStr for PermissionEffect {
    type Err = AccessControlDomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            _ => Err(AccessControlDomainError::InfrastructureError(
                "invalid effect stored".to_string(),
            )),
        }
    }
}
