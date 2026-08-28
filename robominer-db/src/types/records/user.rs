#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub achievement_points: i32,
    pub mining_queue_size: i32,
    pub session_version: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOreAssetStateRecord {
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub max_allowed: i32,
    pub depot_max_allowed: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAssetSummaryRecord {
    pub username: String,
    pub achievement_points: i32,
    pub mining_queue_size: i32,
    pub robot_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppShellHudRecord {
    pub ore_assets: Vec<UserOreAssetStateRecord>,
    pub queue_used: i64,
    pub queue_capacity: i64,
    pub claimable_achievements_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserOreMinedRecord {
    pub ore_id: i64,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserDepotTotalRecord {
    pub ore_id: i64,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UserMiningAreaScoreRecord {
    pub mining_area_id: i64,
    pub score: f64,
}
