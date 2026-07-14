use crate::output::escape_state_field;
use anyhow::{Context, Result};

pub(crate) async fn leaderboard_states(
    pool: &robominer_db::MySqlPool,
    max_entries: i64,
) -> Result<()> {
    let mining_areas = robominer_domain::list_leaderboard_mining_areas(pool)
        .await
        .context("failed to load leaderboard mining areas")?;
    let mining_area_scores =
        robominer_domain::list_leaderboard_mining_area_scores(pool, max_entries)
            .await
            .context("failed to load leaderboard mining area scores")?;
    let top_robots = robominer_domain::list_leaderboard_top_robots(pool, max_entries)
        .await
        .context("failed to load leaderboard top robots")?;
    let top_users = robominer_domain::list_leaderboard_top_users(pool, max_entries)
        .await
        .context("failed to load leaderboard top users")?;

    for mining_area in mining_areas {
        println!(
            "A\t{}\t{}",
            mining_area.id,
            escape_state_field(&mining_area.area_name)
        );
    }

    for score in mining_area_scores {
        println!(
            "S\t{}\t{}\t{}\t{}\t{}",
            score.mining_area_id,
            escape_state_field(&score.robot_name),
            escape_state_field(&score.username),
            score.score,
            score.total_runs
        );
    }

    for robot in top_robots {
        println!(
            "R\t{}\t{}\t{}",
            escape_state_field(&robot.robot_name),
            escape_state_field(&robot.username),
            robot.ore_per_run
        );
    }

    for user in top_users {
        println!(
            "U\t{}\t{}",
            escape_state_field(&user.username),
            user.achievement_points
        );
    }

    Ok(())
}
