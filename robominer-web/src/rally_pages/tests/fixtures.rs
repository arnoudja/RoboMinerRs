//! Shared fixtures for `rally_pages` unit tests.

use std::collections::HashMap;

use crate::Request;

use super::super::{
    ACTIVITY_RALLY_PAGE_SIZE, ActivityFeedQuery, ActivityPageState, ActivityRallyFilter,
    RallyViewPageState,
};

pub(super) fn sample_activity_state(
    recent_users: Vec<robominer_db::ActivityRecentUserRecord>,
    recent_rallies: Vec<robominer_db::ActivityRecentRallyRecord>,
    participants: Vec<robominer_db::ActivityRecentRallyParticipantRecord>,
    rally_areas: Vec<robominer_db::ActivityRallyAreaOption>,
    has_more_rallies: bool,
) -> ActivityPageState {
    sample_activity_state_with_queue(
        recent_users,
        recent_rallies,
        participants,
        rally_areas,
        has_more_rallies,
        vec![],
        None,
    )
}

pub(super) fn sample_activity_state_with_queue(
    recent_users: Vec<robominer_db::ActivityRecentUserRecord>,
    recent_rallies: Vec<robominer_db::ActivityRecentRallyRecord>,
    participants: Vec<robominer_db::ActivityRecentRallyParticipantRecord>,
    rally_areas: Vec<robominer_db::ActivityRallyAreaOption>,
    has_more_rallies: bool,
    queue_items: Vec<robominer_db::MiningQueuePageItemRecord>,
    asset_summary: Option<robominer_db::UserAssetSummaryRecord>,
) -> ActivityPageState {
    ActivityPageState {
        recent_users,
        recent_rallies,
        participants,
        rally_areas,
        has_more_rallies,
        queue_items,
        asset_summary,
    }
}

pub(super) fn default_activity_feed_query() -> ActivityFeedQuery {
    ActivityFeedQuery {
        filter: ActivityRallyFilter::All,
        area_id: None,
        limit: ACTIVITY_RALLY_PAGE_SIZE,
    }
}

pub(super) fn sample_rally_view_state(slots: [(String, String); 4]) -> RallyViewPageState {
    RallyViewPageState {
        result_data: r#"{"v":1,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#.to_string(),
        ores: Vec::new(),
        slots,
        mining_area_name: "Area & One".to_string(),
        viewer_player_number: None,
        viewer_robot_id: None,
        viewer_robot_name: None,
        viewer_score: None,
        viewer_total_reward: None,
        viewer_result_claimed: false,
        viewer_source_code: None,
        viewer_program_source_id: None,
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
