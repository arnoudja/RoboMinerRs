#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceVerification {
    pub verified: bool,
    pub compiled_size: i32,
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceRecord {
    pub id: i64,
    pub user_id: i64,
    pub source_name: String,
    pub source_code: Option<String>,
    pub verified: bool,
    pub compiled_size: i32,
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceStateRecord {
    pub source: ProgramSourceRecord,
    pub linked_robot_count: i64,
}
