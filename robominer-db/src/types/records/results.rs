#[derive(Debug, Clone, PartialEq)]
pub struct MiningResultStateRecord {
    pub robot_id: i64,
    pub mining_queue_id: i64,
    pub mining_area_id: i64,
    pub mining_area_name: String,
    pub rally_result_id: Option<i64>,
    pub score: f64,
    pub score_ore_target: i32,
    pub total_ore_mined: i32,
    pub total_tax: i32,
    pub total_reward: i32,
    pub creation_time_millis: i64,
    pub mining_end_time_millis: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningResultOreStateRecord {
    pub mining_queue_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub tax: i32,
    pub reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningResultActionStateRecord {
    pub mining_queue_id: i64,
    pub action_type: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningResultAreaOreRecord {
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
}
