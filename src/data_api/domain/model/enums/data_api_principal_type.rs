#[derive(Clone, Copy, Debug)]
pub enum DataApiPrincipalType {
    ApiKey,
    Jwt,
}

impl DataApiPrincipalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::Jwt => "jwt",
        }
    }
}
