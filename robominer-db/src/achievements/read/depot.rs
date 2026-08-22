use sqlx::MySqlPool;

use crate::UserDepotTotalRecord;

const USER_DEPOT_TOTAL_FOR_ORE_SQL: &str = "SELECT CAST(COALESCE(SUM(MiningOreResult.depotAmount), 0) AS SIGNED) \
     FROM MiningOreResult \
     INNER JOIN MiningQueue ON MiningQueue.id = MiningOreResult.miningQueueId \
     INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
     WHERE Robot.userId = ? AND MiningOreResult.oreId = ? AND MiningQueue.claimed = true";

pub(crate) async fn user_depot_total_for_ore<'e, E>(
    executor: E,
    user_id: i64,
    ore_id: i64,
) -> Result<i32, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    sqlx::query_scalar(USER_DEPOT_TOTAL_FOR_ORE_SQL)
        .bind(user_id)
        .bind(ore_id)
        .fetch_one(executor)
        .await
}

pub async fn list_user_depot_totals(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserDepotTotalRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32)>(
        "SELECT MiningOreResult.oreId, \
                CAST(COALESCE(SUM(MiningOreResult.depotAmount), 0) AS SIGNED) \
         FROM MiningOreResult \
         INNER JOIN MiningQueue ON MiningQueue.id = MiningOreResult.miningQueueId \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE Robot.userId = ? AND MiningQueue.claimed = true \
         GROUP BY MiningOreResult.oreId \
         ORDER BY MiningOreResult.oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(ore_id, amount)| UserDepotTotalRecord { ore_id, amount })
            .collect()
    })
}
