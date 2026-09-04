use sqlx::MySqlPool;

use crate::{RallyViewMetadataRecord, RallyViewParticipantRecord, RallyViewStateRecord};

#[derive(sqlx::FromRow)]
struct RallyViewStateRow {
    #[sqlx(rename = "resultData")]
    result_data: String,
    #[sqlx(rename = "aiRobotName")]
    ai_robot_name: String,
    #[sqlx(rename = "aiUsername")]
    ai_username: String,
}

impl From<RallyViewStateRow> for RallyViewStateRecord {
    fn from(row: RallyViewStateRow) -> Self {
        Self {
            result_data: row.result_data,
            ai_robot_name: row.ai_robot_name,
            ai_username: row.ai_username,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RallyViewViewerContextRow {
    #[sqlx(rename = "playerNumber")]
    player_number: i32,
    #[sqlx(rename = "robotId")]
    robot_id: i64,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    claimed: bool,
    score: f64,
    #[sqlx(rename = "totalOreMined")]
    total_ore_mined: i32,
    #[sqlx(rename = "totalTax")]
    total_tax: i32,
    #[sqlx(rename = "totalReward")]
    total_reward: i32,
}

#[derive(sqlx::FromRow)]
struct RallyViewParticipantRow {
    #[sqlx(rename = "playerNumber")]
    player_number: i32,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    username: String,
}

impl From<RallyViewParticipantRow> for RallyViewParticipantRecord {
    fn from(row: RallyViewParticipantRow) -> Self {
        Self {
            player_number: row.player_number,
            robot_name: row.robot_name,
            username: row.username,
        }
    }
}

pub async fn rally_view_state(
    pool: &MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_user_result: bool,
) -> Result<Option<RallyViewStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, RallyViewStateRow>(
        "SELECT RallyResult.resultData AS resultData, AiRobot.robotName AS aiRobotName, \
                'AI' AS aiUsername \
         FROM RallyResult \
         INNER JOIN MiningQueue ON MiningQueue.rallyResultId = RallyResult.id \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         INNER JOIN AIRobot AiRobot ON AiRobot.id = MiningArea.aiRobotId \
         WHERE RallyResult.id = ? \
           AND (? = 0 OR EXISTS (SELECT 1 \
                                 FROM MiningQueue UserQueue \
                                 INNER JOIN Robot UserRobot ON UserRobot.id = UserQueue.robotId \
                                 WHERE UserQueue.rallyResultId = RallyResult.id \
                                   AND UserQueue.claimed = TRUE \
                                   AND UserRobot.userId = ?)) \
         ORDER BY MiningQueue.playerNumber, MiningQueue.id \
         LIMIT 1",
    )
    .bind(rally_result_id)
    .bind(if require_user_result { 1_i32 } else { 0_i32 })
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(RallyViewStateRecord::from))
}

pub async fn rally_view_metadata(
    pool: &MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_claimed_viewer_result: bool,
) -> Result<Option<RallyViewMetadataRecord>, sqlx::Error> {
    let Some((mining_area_id, mining_area_name)) = sqlx::query_as::<_, (i64, String)>(
        "SELECT MiningArea.id, MiningArea.areaName \
         FROM MiningQueue \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         WHERE MiningQueue.rallyResultId = ? \
         ORDER BY MiningQueue.playerNumber, MiningQueue.id \
         LIMIT 1",
    )
    .bind(rally_result_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let viewer_context = if user_id > 0 {
        sqlx::query_as::<_, RallyViewViewerContextRow>(
            "SELECT MiningQueue.playerNumber AS playerNumber, MiningQueue.robotId AS robotId, \
                    Robot.robotName AS robotName, MiningQueue.claimed AS claimed, \
                    COALESCE(MiningQueue.score, 0.0) AS score, \
                    CAST(COALESCE(SUM(MiningOreResult.amount), 0) AS SIGNED) AS totalOreMined, \
                    CAST(COALESCE(SUM(COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED) AS totalTax, \
                    CAST(COALESCE(SUM(MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED) \
                      AS totalReward \
             FROM MiningQueue \
             INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
             LEFT OUTER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
             WHERE MiningQueue.rallyResultId = ? \
               AND Robot.userId = ? \
               AND (? = 0 OR MiningQueue.claimed = TRUE) \
             GROUP BY MiningQueue.playerNumber, MiningQueue.robotId, Robot.robotName, \
                      MiningQueue.claimed, MiningQueue.score, MiningQueue.id \
             ORDER BY MiningQueue.playerNumber, MiningQueue.id \
             LIMIT 1",
        )
        .bind(rally_result_id)
        .bind(user_id)
        .bind(if require_claimed_viewer_result {
            1_i32
        } else {
            0_i32
        })
        .fetch_optional(pool)
        .await?
    } else {
        None
    };

    let (
        viewer_player_number,
        viewer_robot_id,
        viewer_robot_name,
        viewer_score,
        viewer_total_ore_mined,
        viewer_total_tax,
        viewer_total_reward,
        viewer_result_claimed,
    ) = if let Some(row) = viewer_context {
        (
            Some(row.player_number),
            Some(row.robot_id),
            Some(row.robot_name),
            Some(row.score),
            Some(row.total_ore_mined),
            Some(row.total_tax),
            Some(row.total_reward),
            row.claimed,
        )
    } else {
        (None, None, None, None, None, None, None, false)
    };

    Ok(Some(RallyViewMetadataRecord {
        mining_area_id,
        mining_area_name,
        viewer_player_number,
        viewer_robot_id,
        viewer_robot_name,
        viewer_score,
        viewer_total_ore_mined,
        viewer_total_tax,
        viewer_total_reward,
        viewer_result_claimed,
    }))
}

pub async fn list_rally_view_participants(
    pool: &MySqlPool,
    rally_result_id: i64,
) -> Result<Vec<RallyViewParticipantRecord>, sqlx::Error> {
    sqlx::query_as::<_, RallyViewParticipantRow>(
        "SELECT MiningQueue.playerNumber AS playerNumber, Robot.robotName AS robotName, \
                User.username AS username \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE MiningQueue.rallyResultId = ? \
           AND MiningQueue.playerNumber IS NOT NULL \
         ORDER BY MiningQueue.playerNumber, MiningQueue.id",
    )
    .bind(rally_result_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(RallyViewParticipantRecord::from)
            .collect()
    })
}

/// Private program snapshot for the viewing user's queue entry in this rally.
pub async fn rally_view_executed_source_code(
    pool: &MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    robot_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT MiningQueue.executedSourceCode \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE MiningQueue.rallyResultId = ? \
           AND MiningQueue.robotId = ? \
           AND Robot.userId = ? \
         ORDER BY MiningQueue.playerNumber, MiningQueue.id \
         LIMIT 1",
    )
    .bind(rally_result_id)
    .bind(robot_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.flatten())
}
