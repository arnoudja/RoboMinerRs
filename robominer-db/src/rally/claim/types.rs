#[derive(Debug, Clone, Copy)]
pub(super) struct ClaimableMiningQueue {
    pub mining_queue_id: i64,
    pub mining_area_id: i64,
    pub robot_id: i64,
    pub robot_max_ore: i32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ClaimableMiningOreResult {
    pub mining_queue_id: i64,
    pub ore_id: i64,
    pub amount: i32,
    pub tax: i32,
}
