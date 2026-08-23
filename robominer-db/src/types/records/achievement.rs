#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AchievementClaimStateRecord {
    pub achievement_id: i64,
    pub claimable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AchievementPagePointsSummaryRecord {
    pub points_earned: i64,
    pub points_achievable: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementOverviewTrackRecord {
    pub achievement_id: i64,
    pub title: String,
    pub description: String,
    pub steps_claimed: i32,
    pub number_of_steps: i64,
    pub points_earned: i64,
    pub total_points: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AchievementPageStateRecord {
    pub achievement_id: i64,
    pub title: String,
    pub description: String,
    pub steps_claimed: i32,
    pub number_of_steps: i64,
    pub achievement_points_earned: i64,
    pub total_achievement_points: i64,
    pub step: i32,
    pub next_achievement_points: i32,
    pub mining_queue_reward: i32,
    pub robot_reward: i32,
    pub ore_id: Option<i64>,
    pub ore_name: Option<String>,
    pub current_ore_maximum: i32,
    pub max_ore_reward: i32,
    pub current_depot_maximum: i32,
    pub max_depot_reward: i32,
    pub mining_area_id: Option<i64>,
    pub mining_area_name: Option<String>,
    pub claimable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementPageTotalRequirementRecord {
    pub achievement_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub current_amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementPageDepotTotalRequirementRecord {
    pub achievement_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub current_amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AchievementPageScoreRequirementRecord {
    pub achievement_id: i64,
    pub mining_area_id: i64,
    pub area_name: String,
    pub minimum_score: f64,
    pub current_score: f64,
    pub current_score_robot_name: Option<String>,
}
