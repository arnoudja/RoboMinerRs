use crate::{Request, query_i64};

pub(super) const ACTIVITY_RALLY_PAGE_SIZE: i64 = 10;
pub(super) const ACTIVITY_RALLY_MAX_LIMIT: i64 = 50;
pub(super) const ACTIVITY_RALLY_MAX_AREAS: i64 = 50;
pub(super) const ACTIVITY_SIDEBAR_QUEUE_PREVIEW: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RallyViewBackLink<'a> {
    MiningResults(&'a str),
    Activity(ActivityFeedQuery),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ActivityRallyFilter {
    All,
    Mine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ActivityFeedQuery {
    filter: ActivityRallyFilter,
    area_id: Option<i64>,
    limit: i64,
}

impl ActivityRallyFilter {
    fn from_request(request: &Request) -> Self {
        match request.query.get("filter").map(String::as_str) {
            Some("mine") => Self::Mine,
            _ => Self::All,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All rallies",
            Self::Mine => "Your rallies",
        }
    }
}

impl ActivityFeedQuery {
    fn from_request(request: &Request) -> Self {
        let filter = ActivityRallyFilter::from_request(request);
        let area_id = query_i64(request, "areaId");
        let limit = query_i64(request, "limit")
            .unwrap_or(ACTIVITY_RALLY_PAGE_SIZE)
            .clamp(ACTIVITY_RALLY_PAGE_SIZE, ACTIVITY_RALLY_MAX_LIMIT);
        Self {
            filter,
            area_id,
            limit,
        }
    }

    fn path_with_query(self, limit: i64) -> String {
        let mut parts = Vec::new();
        if self.filter == ActivityRallyFilter::Mine {
            parts.push("filter=mine".to_string());
        }
        if let Some(area_id) = self.area_id {
            parts.push(format!("areaId={area_id}"));
        }
        if limit != ACTIVITY_RALLY_PAGE_SIZE {
            parts.push(format!("limit={limit}"));
        }
        if parts.is_empty() {
            "activity".to_string()
        } else {
            format!("activity?{}", parts.join("&"))
        }
    }

    fn href(self) -> String {
        self.path_with_query(self.limit)
    }

    fn filter_href(self, filter: ActivityRallyFilter) -> String {
        Self {
            filter,
            area_id: self.area_id,
            limit: ACTIVITY_RALLY_PAGE_SIZE,
        }
        .href()
    }

    fn area_href(self, area_id: Option<i64>) -> String {
        Self {
            area_id,
            limit: ACTIVITY_RALLY_PAGE_SIZE,
            ..self
        }
        .href()
    }

    fn load_more_href(self) -> String {
        let next_limit = (self.limit + ACTIVITY_RALLY_PAGE_SIZE).min(ACTIVITY_RALLY_MAX_LIMIT);
        self.path_with_query(next_limit)
    }

    fn append_to_href(self, href: &str) -> String {
        let query = self.href();
        if query == "activity" {
            return href.to_string();
        }
        let suffix = query.strip_prefix("activity?").unwrap_or("");
        if href.contains('?') {
            format!("{href}&{suffix}")
        } else {
            format!("{href}?{suffix}")
        }
    }
}

#[derive(Debug)]
pub(super) struct ActivityPageState {
    recent_users: Vec<robominer_db::ActivityRecentUserRecord>,
    recent_rallies: Vec<robominer_db::ActivityRecentRallyRecord>,
    participants: Vec<robominer_db::ActivityRecentRallyParticipantRecord>,
    rally_areas: Vec<robominer_db::ActivityRallyAreaOption>,
    has_more_rallies: bool,
    queue_items: Vec<robominer_db::MiningQueuePageItemRecord>,
    asset_summary: Option<robominer_db::UserAssetSummaryRecord>,
}

#[derive(Debug)]
pub(super) struct RallyViewPageState {
    pub(super) result_data: String,
    pub(super) ores: Vec<robominer_db::OreRecord>,
    pub(super) slots: [(String, String); 4],
    pub(super) mining_area_name: String,
    pub(super) viewer_player_number: Option<i32>,
    pub(super) viewer_robot_id: Option<i64>,
    pub(super) viewer_robot_name: Option<String>,
    pub(super) viewer_score: Option<f64>,
    pub(super) viewer_total_reward: Option<i32>,
    pub(super) viewer_result_claimed: bool,
}

mod activity;
mod view;

pub(super) use activity::activity_page;
#[allow(unused_imports)]
pub(super) use activity::render::{render_activity_page, render_activity_page_at};
pub(super) use view::render::render_rally_view_page;
pub(super) use view::{load_user_rally_view_state, valid_mining_results_return_to};

#[cfg(test)]
mod tests;
