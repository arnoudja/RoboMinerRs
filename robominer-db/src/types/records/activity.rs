#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRecentUserRecord {
    pub user_id: i64,
    pub username: String,
    pub last_login_time_millis: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityRecentRallyRecord {
    pub mining_queue_id: i64,
    pub rally_result_id: Option<i64>,
    pub mining_area_id: i64,
    pub mining_area_name: String,
    pub robot_name: String,
    pub username: String,
    pub score: f64,
    pub mining_end_time_millis: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRallyAreaOption {
    pub mining_area_id: i64,
    pub area_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityRecentRallyParticipantRecord {
    pub mining_queue_id: i64,
    pub player_number: i32,
    pub robot_name: String,
    pub username: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RallyViewStateRecord {
    pub result_data: String,
    pub ai_robot_name: String,
    pub ai_username: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RallyViewParticipantRecord {
    pub player_number: i32,
    pub robot_name: String,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RallyViewMetadataRecord {
    pub mining_area_id: i64,
    pub mining_area_name: String,
    pub viewer_player_number: Option<i32>,
    pub viewer_robot_id: Option<i64>,
    pub viewer_robot_name: Option<String>,
    pub viewer_score: Option<f64>,
    pub viewer_total_ore_mined: Option<i32>,
    pub viewer_total_tax: Option<i32>,
    pub viewer_total_reward: Option<i32>,
    pub viewer_result_claimed: bool,
}
