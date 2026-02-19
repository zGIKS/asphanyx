#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvisionedDatabaseStatus {
    Provisioning,
    Active,
    Failed,
    Deleting,
    Deleted,
}

impl ProvisionedDatabaseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Provisioning => "provisioning",
            Self::Active => "active",
            Self::Failed => "failed",
            Self::Deleting => "deleting",
            Self::Deleted => "deleted",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "provisioning" => Some(Self::Provisioning),
            "active" => Some(Self::Active),
            "failed" => Some(Self::Failed),
            "deleting" => Some(Self::Deleting),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }
}
