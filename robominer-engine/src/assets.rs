use crate::output::escape_state_field;
use anyhow::{Context, Result};

pub(crate) async fn user_ore_asset_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<()> {
    let summary = robominer_db::load_user_asset_summary(pool, user_id)
        .await
        .context("failed to load user asset summary")?;
    let states = robominer_db::list_user_ore_asset_states(pool, user_id)
        .await
        .context("failed to load user ore asset states")?;

    println!(
        "U\t{}\t{}\t{}\t{}",
        escape_state_field(&summary.username),
        summary.achievement_points,
        summary.robot_count,
        summary.mining_queue_size
    );

    for state in states {
        println!(
            "O\t{}\t{}\t{}\t{}",
            state.ore_id,
            escape_state_field(&state.ore_name),
            state.amount,
            state.max_allowed
        );
    }

    Ok(())
}
