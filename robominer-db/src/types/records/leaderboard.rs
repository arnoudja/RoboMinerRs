#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardMiningAreaRecord {
    pub id: i64,
    pub area_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardMiningAreaScoreRecord {
    pub mining_area_id: i64,
    pub robot_name: String,
    pub username: String,
    pub score: f64,
    pub total_runs: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardTopRobotRecord {
    pub robot_id: i64,
    pub robot_name: String,
    pub username: String,
    pub ore_per_run: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardTopUserRecord {
    pub username: String,
    pub achievement_points: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardViewerStandingRecord {
    pub achievement_points: i32,
    pub achievement_rank: i64,
    pub area_standings: Vec<LeaderboardViewerAreaStandingRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardViewerAreaStandingRecord {
    pub mining_area_id: i64,
    pub area_name: String,
    pub robot_name: String,
    pub score: f64,
    pub rank: i64,
}
