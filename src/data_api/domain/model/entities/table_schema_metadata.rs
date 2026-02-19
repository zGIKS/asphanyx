#[derive(Clone, Debug)]
pub struct TableColumnMetadata {
    pub column_name: String,
    pub is_nullable: bool,
    pub data_type: String,
    pub is_primary_key: bool,
}

#[derive(Clone, Debug)]
pub struct TableSchemaMetadata {
    pub schema_name: String,
    pub table_name: String,
    pub columns: Vec<TableColumnMetadata>,
}

impl TableSchemaMetadata {
    pub fn primary_key_column(&self) -> Option<&TableColumnMetadata> {
        self.columns.iter().find(|c| c.is_primary_key)
    }

    pub fn has_column(&self, column_name: &str) -> bool {
        self.columns.iter().any(|c| c.column_name == column_name)
    }
}
