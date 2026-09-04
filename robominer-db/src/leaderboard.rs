use sqlx::MySqlPool;

use crate::{
    LeaderboardMiningAreaRecord, LeaderboardMiningAreaScoreRecord, LeaderboardTopRobotRecord,
    LeaderboardTopUserRecord, LeaderboardViewerAreaStandingRecord, LeaderboardViewerStandingRecord,
};

const LEADERBOARD_VIEWER_AREA_STANDINGS: i64 = 5;

#[derive(sqlx::FromRow)]
struct LeaderboardMiningAreaScoreRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    username: String,
    score: f64,
    #[sqlx(rename = "totalRuns")]
    total_runs: i32,
}

impl From<LeaderboardMiningAreaScoreRow> for LeaderboardMiningAreaScoreRecord {
    fn from(row: LeaderboardMiningAreaScoreRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            robot_name: row.robot_name,
            username: row.username,
            score: row.score,
            total_runs: row.total_runs,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LeaderboardTopRobotRow {
    #[sqlx(rename = "id")]
    robot_id: i64,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    username: String,
    #[sqlx(rename = "orePerRun")]
    ore_per_run: f64,
}

impl From<LeaderboardTopRobotRow> for LeaderboardTopRobotRecord {
    fn from(row: LeaderboardTopRobotRow) -> Self {
        Self {
            robot_id: row.robot_id,
            robot_name: row.robot_name,
            username: row.username,
            ore_per_run: row.ore_per_run,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LeaderboardViewerAreaStandingRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "areaName")]
    area_name: String,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    score: f64,
    #[sqlx(rename = "areaRank")]
    rank: i64,
}

impl From<LeaderboardViewerAreaStandingRow> for LeaderboardViewerAreaStandingRecord {
    fn from(row: LeaderboardViewerAreaStandingRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            area_name: row.area_name,
            robot_name: row.robot_name,
            score: row.score,
            rank: row.rank,
        }
    }
}

pub async fn list_leaderboard_mining_areas(
    pool: &MySqlPool,
) -> Result<Vec<LeaderboardMiningAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, areaName \
         FROM MiningArea \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(id, area_name)| LeaderboardMiningAreaRecord { id, area_name })
            .collect()
    })
}

pub async fn list_leaderboard_mining_area_scores(
    pool: &MySqlPool,
    maximum_results: i64,
) -> Result<Vec<LeaderboardMiningAreaScoreRecord>, sqlx::Error> {
    sqlx::query_as::<_, LeaderboardMiningAreaScoreRow>(
        "SELECT Score.miningAreaId AS miningAreaId, Robot.robotName AS robotName, \
                User.username AS username, Score.score AS score, Score.totalRuns AS totalRuns \
         FROM RobotMiningAreaScore Score \
         INNER JOIN Robot ON Robot.id = Score.robotId \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE (SELECT COUNT(*) \
                FROM RobotMiningAreaScore RankScore \
                WHERE RankScore.miningAreaId = Score.miningAreaId \
                  AND (RankScore.score > Score.score \
                       OR (RankScore.score = Score.score AND RankScore.robotId <= Score.robotId))) <= ? \
         ORDER BY Score.miningAreaId, Score.score DESC, Score.robotId",
    )
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(LeaderboardMiningAreaScoreRecord::from)
            .collect()
    })
}

pub async fn list_leaderboard_top_robots(
    pool: &MySqlPool,
    maximum_results: i64,
) -> Result<Vec<LeaderboardTopRobotRecord>, sqlx::Error> {
    sqlx::query_as::<_, LeaderboardTopRobotRow>(
        "SELECT Robot.id AS id, Robot.robotName AS robotName, User.username AS username, \
                CAST(COALESCE(SUM(RobotLifetimeResult.amount), 0) / Robot.totalMiningRuns AS DOUBLE) AS orePerRun \
         FROM Robot \
         INNER JOIN User ON User.id = Robot.userId \
         LEFT OUTER JOIN RobotLifetimeResult ON RobotLifetimeResult.robotId = Robot.id \
         WHERE Robot.totalMiningRuns > 0 \
         GROUP BY Robot.id, Robot.robotName, User.username, Robot.totalMiningRuns \
         ORDER BY orePerRun DESC, Robot.id \
         LIMIT ?",
    )
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(LeaderboardTopRobotRecord::from)
            .collect()
    })
}

pub async fn list_leaderboard_top_users(
    pool: &MySqlPool,
    maximum_results: i64,
) -> Result<Vec<LeaderboardTopUserRecord>, sqlx::Error> {
    sqlx::query_as::<_, (String, i32)>(
        "SELECT username, achievementPoints \
         FROM User \
         WHERE id > 1 \
         ORDER BY achievementPoints DESC, id \
         LIMIT ?",
    )
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(username, achievement_points)| LeaderboardTopUserRecord {
                username,
                achievement_points,
            })
            .collect()
    })
}

pub async fn load_leaderboard_viewer_standing(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<LeaderboardViewerStandingRecord, sqlx::Error> {
    let (achievement_points, achievement_rank) = sqlx::query_as::<_, (i32, i64)>(
        "SELECT achievementPoints, \
                (SELECT COUNT(*) + 1 \
                 FROM User RankUser \
                 WHERE RankUser.id > 1 \
                   AND (RankUser.achievementPoints > User.achievementPoints \
                        OR (RankUser.achievementPoints = User.achievementPoints \
                            AND RankUser.id < User.id))) \
         FROM User \
         WHERE User.id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let area_rows = sqlx::query_as::<_, LeaderboardViewerAreaStandingRow>(
        "SELECT Score.miningAreaId AS miningAreaId, MiningArea.areaName AS areaName, \
                Robot.robotName AS robotName, Score.score AS score, \
                (SELECT COUNT(*) + 1 \
                 FROM RobotMiningAreaScore RankScore \
                 WHERE RankScore.miningAreaId = Score.miningAreaId \
                   AND (RankScore.score > Score.score \
                        OR (RankScore.score = Score.score AND RankScore.robotId < Score.robotId))) AS areaRank \
         FROM RobotMiningAreaScore Score \
         INNER JOIN Robot ON Robot.id = Score.robotId \
         INNER JOIN MiningArea ON MiningArea.id = Score.miningAreaId \
         WHERE Robot.userId = ? \
           AND NOT EXISTS ( \
               SELECT 1 \
               FROM RobotMiningAreaScore UserBestCandidate \
               INNER JOIN Robot UserBestRobot ON UserBestRobot.id = UserBestCandidate.robotId \
               WHERE UserBestRobot.userId = ? \
                 AND UserBestCandidate.miningAreaId = Score.miningAreaId \
                 AND (UserBestCandidate.score > Score.score \
                      OR (UserBestCandidate.score = Score.score \
                          AND UserBestCandidate.robotId < Score.robotId)) \
           ) \
         ORDER BY areaRank ASC, Score.score DESC, Score.robotId \
         LIMIT ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(LEADERBOARD_VIEWER_AREA_STANDINGS)
    .fetch_all(pool)
    .await?;

    Ok(LeaderboardViewerStandingRecord {
        achievement_points,
        achievement_rank,
        area_standings: area_rows
            .into_iter()
            .map(LeaderboardViewerAreaStandingRecord::from)
            .collect(),
    })
}
