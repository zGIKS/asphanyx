use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AuthenticatedUserId(Uuid);

impl AuthenticatedUserId {
    pub fn new(value: &str) -> Result<Self, String> {
        let parsed =
            Uuid::parse_str(value).map_err(|_| "subject_id must be a valid UUID".to_string())?;
        Ok(Self(parsed))
    }

    pub fn value(&self) -> Uuid {
        self.0
    }

    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}
