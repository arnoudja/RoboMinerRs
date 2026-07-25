use sqlx::MySqlPool;

use crate::{UserMiningAreaScoreRecord, UserOreMinedRecord};

pub async fn list_user_ore_mined_totals(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserOreMinedRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32)>(
        "SELECT RobotLifetimeResult.oreId, \
                CAST(COALESCE(SUM(RobotLifetimeResult.amount), 0) AS SIGNED) \
         FROM RobotLifetimeResult \
         INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
         WHERE Robot.userId = ? \
         GROUP BY RobotLifetimeResult.oreId \
         ORDER BY RobotLifetimeResult.oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(ore_id, amount)| UserOreMinedRecord { ore_id, amount })
            .collect()
    })
}

pub async fn list_user_best_mining_area_scores(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserMiningAreaScoreRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, f64)>(
        "SELECT RobotMiningAreaScore.miningAreaId, \
                COALESCE(MAX(RobotMiningAreaScore.score), 0.0) \
         FROM RobotMiningAreaScore \
         INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
         WHERE Robot.userId = ? \
         GROUP BY RobotMiningAreaScore.miningAreaId \
         ORDER BY RobotMiningAreaScore.miningAreaId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(mining_area_id, score)| UserMiningAreaScoreRecord {
                mining_area_id,
                score,
            })
            .collect()
    })
}
