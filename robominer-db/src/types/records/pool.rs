#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolRecord {
    pub id: i64,
    pub mining_area_id: i64,
    pub required_runs: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolItemRecord {
    pub id: i64,
    pub pool_id: i64,
    pub robot_id: i64,
    pub source_code: String,
    pub total_score: f64,
    pub runs_done: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedPoolRallyRecord {
    pub items: Vec<CompletedPoolItemRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedPoolItemRecord {
    pub pool_item_id: i64,
    pub score: f64,
    pub ore_results: Vec<CompletedPoolItemOreRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedPoolItemOreRecord {
    pub ore_id: i64,
    pub amount: i32,
}
