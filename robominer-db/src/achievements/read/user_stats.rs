use sqlx::MySqlPool;

use crate::{UserMiningAreaScoreRecord, UserOreMinedRecord};

#[derive(sqlx::FromRow)]
struct UserOreMinedRow {
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    amount: i32,
}

impl From<UserOreMinedRow> for UserOreMinedRecord {
    fn from(row: UserOreMinedRow) -> Self {
        Self {
            ore_id: row.ore_id,
            amount: row.amount,
        }
    }
}

#[derive(sqlx::FromRow)]
struct UserMiningAreaScoreRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    score: f64,
}

impl From<UserMiningAreaScoreRow> for UserMiningAreaScoreRecord {
    fn from(row: UserMiningAreaScoreRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            score: row.score,
        }
    }
}

pub async fn list_user_ore_mined_totals(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserOreMinedRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserOreMinedRow>(
        "SELECT RobotLifetimeResult.oreId AS oreId, \
                CAST(COALESCE(SUM(RobotLifetimeResult.amount), 0) AS SIGNED) AS amount \
         FROM RobotLifetimeResult \
         INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
         WHERE Robot.userId = ? \
         GROUP BY RobotLifetimeResult.oreId \
         ORDER BY RobotLifetimeResult.oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(UserOreMinedRecord::from).collect())
}

pub async fn list_user_best_mining_area_scores(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserMiningAreaScoreRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserMiningAreaScoreRow>(
        "SELECT RobotMiningAreaScore.miningAreaId AS miningAreaId, \
                COALESCE(MAX(RobotMiningAreaScore.score), 0.0) AS score \
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
            .map(UserMiningAreaScoreRecord::from)
            .collect()
    })
}
