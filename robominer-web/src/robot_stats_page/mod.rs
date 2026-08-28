use crate::{Request, Response, ServerConfig, query_i64};

pub(super) const ROBOT_STATS_RECENT_RUNS_LIMIT: i64 = 10;

#[derive(Debug)]
pub(super) struct RobotStatsPageState {
    pub(super) robot_not_found: bool,
    pub(super) header: Option<robominer_db::RobotStatsHeaderRecord>,
    pub(super) ore_stats: Vec<robominer_db::RobotLifetimeOreStatRecord>,
    pub(super) area_stats: Vec<robominer_db::RobotMiningAreaStatRecord>,
    pub(super) recent_runs: Vec<robominer_db::MiningResultStateRecord>,
}

impl RobotStatsPageState {
    pub(super) fn total_ore_mined(&self) -> i64 {
        self.ore_stats.iter().map(|ore| i64::from(ore.amount)).sum()
    }

    pub(super) fn total_tax(&self) -> i64 {
        self.ore_stats.iter().map(|ore| i64::from(ore.tax)).sum()
    }

    pub(super) fn ore_per_run(&self) -> Option<f64> {
        let runs = self.header.as_ref()?.total_mining_runs;
        if runs <= 0 {
            return None;
        }
        Some(self.total_ore_mined() as f64 / f64::from(runs))
    }
}

pub(super) async fn robot_stats_page(request: &Request, config: &ServerConfig) -> Response {
    crate::page_context::with_session_page(
        request,
        config,
        "Robot stats require ROBOMINER_DATABASE_URL to be configured",
        |session| async move {
            let robot_id = query_i64(request, "robotId");
            let result = load_robot_stats_state(session.pool, robot_id).await;

            match result {
                Ok(state) => {
                    session
                        .html_with_hud(request, config, |username, hud| {
                            render::render_robot_stats_page(username, hud, &state)
                        })
                        .await
                }
                Err(error) => crate::page_context::page_load_error("robot stats", error),
            }
        },
    )
    .await
}

async fn load_robot_stats_state(
    pool: &robominer_db::MySqlPool,
    robot_id: Option<i64>,
) -> Result<RobotStatsPageState, crate::page_context::PageLoadError> {
    let Some(robot_id) = robot_id else {
        return Ok(empty_not_found_state());
    };

    let Some(header) = robominer_db::load_robot_stats_header(pool, robot_id).await? else {
        return Ok(empty_not_found_state());
    };

    Ok(RobotStatsPageState {
        robot_not_found: false,
        ore_stats: robominer_db::list_robot_lifetime_ore_stats(pool, robot_id).await?,
        area_stats: robominer_db::list_robot_mining_area_stats(pool, robot_id).await?,
        recent_runs: robominer_db::list_mining_result_states_for_robot(
            pool,
            robot_id,
            ROBOT_STATS_RECENT_RUNS_LIMIT,
        )
        .await?,
        header: Some(header),
    })
}

fn empty_not_found_state() -> RobotStatsPageState {
    RobotStatsPageState {
        robot_not_found: true,
        header: None,
        ore_stats: Vec::new(),
        area_stats: Vec::new(),
        recent_runs: Vec::new(),
    }
}

mod render;

#[cfg(test)]
mod tests;
