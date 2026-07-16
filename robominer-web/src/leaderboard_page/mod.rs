use crate::{Request, Response, ServerConfig, query_i64, request_user_id, session_username};

const LEADERBOARD_PAGE_SIZE: i64 = 10;
const LEADERBOARD_MAX_LIMIT: i64 = 50;
const LEADERBOARD_SIDEBAR_AREA_STANDINGS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LeaderboardTab {
    Areas,
    Robots,
    Players,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LeaderboardQuery {
    tab: LeaderboardTab,
    area_id: Option<i64>,
    limit: i64,
}

impl LeaderboardTab {
    fn from_request(request: &Request) -> Self {
        match request.query.get("tab").map(String::as_str) {
            Some("robots") => Self::Robots,
            Some("players") => Self::Players,
            _ => Self::Areas,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Areas => "By area",
            Self::Robots => "Top robots",
            Self::Players => "Top players",
        }
    }
}

impl LeaderboardQuery {
    fn from_request(request: &Request) -> Self {
        let limit = query_i64(request, "limit")
            .unwrap_or(LEADERBOARD_PAGE_SIZE)
            .clamp(LEADERBOARD_PAGE_SIZE, LEADERBOARD_MAX_LIMIT);
        Self {
            tab: LeaderboardTab::from_request(request),
            area_id: query_i64(request, "areaId"),
            limit,
        }
    }

    fn path_with_query(self, tab: LeaderboardTab, area_id: Option<i64>, limit: i64) -> String {
        let mut parts = Vec::new();
        if tab != LeaderboardTab::Areas {
            parts.push(match tab {
                LeaderboardTab::Robots => "tab=robots".to_string(),
                LeaderboardTab::Players => "tab=players".to_string(),
                LeaderboardTab::Areas => unreachable!(),
            });
        }
        if tab == LeaderboardTab::Areas
            && let Some(area_id) = area_id
        {
            parts.push(format!("areaId={area_id}"));
        }
        if limit != LEADERBOARD_PAGE_SIZE {
            parts.push(format!("limit={limit}"));
        }
        if parts.is_empty() {
            "leaderboard".to_string()
        } else {
            format!("leaderboard?{}", parts.join("&"))
        }
    }

    fn tab_href(self, tab: LeaderboardTab) -> String {
        let area_id = if tab == LeaderboardTab::Areas {
            self.area_id
        } else {
            None
        };
        self.path_with_query(tab, area_id, self.limit)
    }

    fn area_href(self, area_id: Option<i64>) -> String {
        self.path_with_query(LeaderboardTab::Areas, area_id, self.limit)
    }

    fn load_more_href(self) -> String {
        let next_limit = (self.limit + LEADERBOARD_PAGE_SIZE).min(LEADERBOARD_MAX_LIMIT);
        self.path_with_query(self.tab, self.area_id, next_limit)
    }

    fn resolved_area_id(
        self,
        ranked_areas: &[&robominer_db::LeaderboardMiningAreaRecord],
    ) -> Option<i64> {
        if self.tab != LeaderboardTab::Areas {
            return None;
        }
        if let Some(area_id) = self.area_id
            && ranked_areas.iter().any(|area| area.id == area_id)
        {
            return Some(area_id);
        }
        ranked_areas.first().map(|area| area.id)
    }
}

#[derive(Debug)]
pub(super) struct LeaderboardPageState {
    mining_areas: Vec<robominer_db::LeaderboardMiningAreaRecord>,
    mining_area_scores: Vec<robominer_db::LeaderboardMiningAreaScoreRecord>,
    top_robots: Vec<robominer_db::LeaderboardTopRobotRecord>,
    top_users: Vec<robominer_db::LeaderboardTopUserRecord>,
    viewer_standing: Option<robominer_db::LeaderboardViewerStandingRecord>,
    has_more_robots: bool,
    has_more_players: bool,
}

pub(super) async fn leaderboard_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Leaderboard requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let user_id = request_user_id(request).unwrap_or(0);
    let query = LeaderboardQuery::from_request(request);
    let result = load_leaderboard_state(pool, user_id, query).await;

    match result {
        Ok(state) => Response::html(render::render_leaderboard_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config)
                .await
                .as_deref(),
            query,
            &state,
        )),
        Err(error) => Response::service_unavailable(format!("Unable to load leaderboard: {error}")),
    }
}

async fn load_leaderboard_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    query: LeaderboardQuery,
) -> Result<LeaderboardPageState, robominer_domain::DomainError> {
    let fetch_limit = query.limit + 1;
    let viewer_standing = if user_id > 0 {
        Some(robominer_db::load_leaderboard_viewer_standing(pool, user_id).await?)
    } else {
        None
    };

    let mut top_robots = robominer_db::list_leaderboard_top_robots(pool, fetch_limit).await?;
    let has_more_robots = top_robots.len() as i64 > query.limit;
    top_robots.truncate(query.limit as usize);

    let mut top_users = robominer_db::list_leaderboard_top_users(pool, fetch_limit).await?;
    let has_more_players = top_users.len() as i64 > query.limit;
    top_users.truncate(query.limit as usize);

    Ok(LeaderboardPageState {
        mining_areas: robominer_db::list_leaderboard_mining_areas(pool).await?,
        mining_area_scores: robominer_db::list_leaderboard_mining_area_scores(pool, fetch_limit)
            .await?,
        top_robots,
        top_users,
        viewer_standing,
        has_more_robots,
        has_more_players,
    })
}

mod render;
mod render_areas;
mod render_players;
mod render_robots;
mod render_shared;
mod render_sidebar;

#[cfg(test)]
mod tests;
