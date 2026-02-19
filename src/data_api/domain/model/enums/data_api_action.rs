#[derive(Clone, Copy, Debug)]
pub enum DataApiAction {
    Create,
    Read,
    Update,
    Delete,
}

impl DataApiAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}
