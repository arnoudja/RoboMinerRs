use sqlx::MySqlPool;

use crate::{
    AppShellHudRecord, list_achievement_claim_states_for_user, list_user_ore_asset_states,
    load_user_asset_summary,
};

pub async fn load_app_shell_hud(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<AppShellHudRecord, sqlx::Error> {
    let summary = load_user_asset_summary(pool, user_id).await?;
    let ore_assets = list_user_ore_asset_states(pool, user_id).await?;
    let queue_used: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE Robot.userId = ? \
           AND (MiningQueue.miningEndTime IS NULL OR MiningQueue.miningEndTime > NOW())",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    let claimable_achievements_count = list_achievement_claim_states_for_user(pool, user_id)
        .await?
        .into_iter()
        .filter(|state| state.claimable)
        .count() as u64;

    let queue_capacity = summary.robot_count * i64::from(summary.mining_queue_size);

    Ok(AppShellHudRecord {
        ore_assets,
        queue_used,
        queue_capacity,
        claimable_achievements_count,
    })
}
