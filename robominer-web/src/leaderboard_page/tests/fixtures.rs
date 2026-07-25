//! Shared fixtures for `leaderboard_page` unit tests.

use std::collections::HashMap;

use crate::Request;

use super::super::{LEADERBOARD_PAGE_SIZE, LeaderboardPageState, LeaderboardQuery, LeaderboardTab};

pub(super) fn areas_leaderboard_query() -> LeaderboardQuery {
    LeaderboardQuery {
        tab: LeaderboardTab::Areas,
        area_id: Some(1),
        limit: LEADERBOARD_PAGE_SIZE,
    }
}

pub(super) fn sample_leaderboard_state(
    mining_areas: Vec<robominer_db::LeaderboardMiningAreaRecord>,
    mining_area_scores: Vec<robominer_db::LeaderboardMiningAreaScoreRecord>,
    top_robots: Vec<robominer_db::LeaderboardTopRobotRecord>,
    top_users: Vec<robominer_db::LeaderboardTopUserRecord>,
    viewer_standing: Option<robominer_db::LeaderboardViewerStandingRecord>,
) -> LeaderboardPageState {
    sample_leaderboard_state_with_more(
        mining_areas,
        mining_area_scores,
        top_robots,
        top_users,
        viewer_standing,
        false,
        false,
    )
}

pub(super) fn sample_leaderboard_state_with_more(
    mining_areas: Vec<robominer_db::LeaderboardMiningAreaRecord>,
    mining_area_scores: Vec<robominer_db::LeaderboardMiningAreaScoreRecord>,
    top_robots: Vec<robominer_db::LeaderboardTopRobotRecord>,
    top_users: Vec<robominer_db::LeaderboardTopUserRecord>,
    viewer_standing: Option<robominer_db::LeaderboardViewerStandingRecord>,
    has_more_robots: bool,
    has_more_players: bool,
) -> LeaderboardPageState {
    LeaderboardPageState {
        mining_areas,
        mining_area_scores,
        top_robots,
        top_users,
        viewer_standing,
        has_more_robots,
        has_more_players,
    }
}

pub(super) fn request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::new(),
    }
}
