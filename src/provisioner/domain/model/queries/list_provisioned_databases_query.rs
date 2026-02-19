#[derive(Clone, Debug, Default)]
pub struct ListProvisionedDatabasesQuery {
    include_deleted: bool,
}

impl ListProvisionedDatabasesQuery {
    pub fn new(include_deleted: bool) -> Self {
        Self { include_deleted }
    }

    pub fn include_deleted(&self) -> bool {
        self.include_deleted
    }
}
