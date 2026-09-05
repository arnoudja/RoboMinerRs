use sqlx::MySqlPool;

use crate::{
    RobotLifetimeOreStatRecord, RobotMiningAreaScoreRecord, RobotMiningAreaStatRecord,
    RobotStatsHeaderRecord,
};

#[derive(sqlx::FromRow)]
struct RobotStatsHeaderRow {
    #[sqlx(rename = "robotId")]
    robot_id: i64,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    username: String,
    #[sqlx(rename = "totalMiningRuns")]
    total_mining_runs: i32,
}

impl From<RobotStatsHeaderRow> for RobotStatsHeaderRecord {
    fn from(row: RobotStatsHeaderRow) -> Self {
        Self {
            robot_id: row.robot_id,
            robot_name: row.robot_name,
            username: row.username,
            total_mining_runs: row.total_mining_runs,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RobotLifetimeOreStatRow {
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "oreName")]
    ore_name: String,
    amount: i32,
    tax: i32,
}

impl From<RobotLifetimeOreStatRow> for RobotLifetimeOreStatRecord {
    fn from(row: RobotLifetimeOreStatRow) -> Self {
        Self {
            ore_id: row.ore_id,
            ore_name: row.ore_name,
            amount: row.amount,
            tax: row.tax,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RobotMiningAreaStatRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "areaName")]
    area_name: String,
    #[sqlx(rename = "totalRuns")]
    total_runs: i32,
    score: f64,
}

impl From<RobotMiningAreaStatRow> for RobotMiningAreaStatRecord {
    fn from(row: RobotMiningAreaStatRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            area_name: row.area_name,
            total_runs: row.total_runs,
            score: row.score,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RobotMiningAreaScoreRow {
    #[sqlx(rename = "robotId")]
    robot_id: i64,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    score: f64,
}

impl From<RobotMiningAreaScoreRow> for RobotMiningAreaScoreRecord {
    fn from(row: RobotMiningAreaScoreRow) -> Self {
        Self {
            robot_id: row.robot_id,
            mining_area_id: row.mining_area_id,
            score: row.score,
        }
    }
}

pub async fn load_robot_stats_header(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Option<RobotStatsHeaderRecord>, sqlx::Error> {
    sqlx::query_as::<_, RobotStatsHeaderRow>(
        "SELECT Robot.id AS robotId, Robot.robotName AS robotName, User.username AS username, \
                Robot.totalMiningRuns AS totalMiningRuns \
         FROM Robot \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE Robot.id = ?",
    )
    .bind(robot_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(RobotStatsHeaderRecord::from))
}

pub async fn list_robot_lifetime_ore_stats(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Vec<RobotLifetimeOreStatRecord>, sqlx::Error> {
    sqlx::query_as::<_, RobotLifetimeOreStatRow>(
        "SELECT Ore.id AS oreId, Ore.oreName AS oreName, RobotLifetimeResult.amount AS amount, \
                RobotLifetimeResult.tax AS tax \
         FROM RobotLifetimeResult \
         INNER JOIN Ore ON Ore.id = RobotLifetimeResult.oreId \
         WHERE RobotLifetimeResult.robotId = ? \
         ORDER BY Ore.id",
    )
    .bind(robot_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(RobotLifetimeOreStatRecord::from)
            .collect()
    })
}

pub async fn list_robot_mining_area_stats(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Vec<RobotMiningAreaStatRecord>, sqlx::Error> {
    sqlx::query_as::<_, RobotMiningAreaStatRow>(
        "SELECT MiningArea.id AS miningAreaId, MiningArea.areaName AS areaName, \
                RobotMiningAreaScore.totalRuns AS totalRuns, RobotMiningAreaScore.score AS score \
         FROM RobotMiningAreaScore \
         INNER JOIN MiningArea ON MiningArea.id = RobotMiningAreaScore.miningAreaId \
         WHERE RobotMiningAreaScore.robotId = ? \
         ORDER BY MiningArea.id",
    )
    .bind(robot_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(RobotMiningAreaStatRecord::from)
            .collect()
    })
}

pub async fn list_robot_mining_area_scores_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<RobotMiningAreaScoreRecord>, sqlx::Error> {
    sqlx::query_as::<_, RobotMiningAreaScoreRow>(
        "SELECT RobotMiningAreaScore.robotId AS robotId, \
                RobotMiningAreaScore.miningAreaId AS miningAreaId, \
                RobotMiningAreaScore.score AS score \
         FROM RobotMiningAreaScore \
         INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
         WHERE Robot.userId = ? \
         ORDER BY RobotMiningAreaScore.robotId, RobotMiningAreaScore.miningAreaId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(RobotMiningAreaScoreRecord::from)
            .collect()
    })
}
pub async fn count_user_robots(pool: &MySqlPool, user_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM Robot WHERE userId = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
}
