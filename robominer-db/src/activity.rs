use sqlx::MySqlPool;

use crate::{
    ActivityRallyAreaOption, ActivityRecentRallyParticipantRecord, ActivityRecentRallyRecord,
    ActivityRecentUserRecord, RallyViewMetadataRecord, RallyViewParticipantRecord,
    RallyViewStateRecord,
};

pub async fn list_activity_recent_users(
    pool: &MySqlPool,
    maximum_users: i64,
) -> Result<Vec<ActivityRecentUserRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i64)>(
        "SELECT id, username, CAST(UNIX_TIMESTAMP(lastLoginTime) * 1000 AS SIGNED) \
         FROM User \
         WHERE id > 1 \
         ORDER BY lastLoginTime DESC \
         LIMIT ?",
    )
    .bind(maximum_users)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(user_id, username, last_login_time_millis)| ActivityRecentUserRecord {
                    user_id,
                    username,
                    last_login_time_millis,
                },
            )
            .collect()
    })
}

pub async fn list_activity_recent_rallies(
    pool: &MySqlPool,
    maximum_rallies: i64,
) -> Result<Vec<ActivityRecentRallyRecord>, sqlx::Error> {
    list_activity_recent_rally_feed(pool, None, None, maximum_rallies)
        .await
        .map(|(rallies, _)| rallies)
}

pub async fn list_activity_recent_rally_feed(
    pool: &MySqlPool,
    user_id: Option<i64>,
    mining_area_id: Option<i64>,
    limit: i64,
) -> Result<(Vec<ActivityRecentRallyRecord>, bool), sqlx::Error> {
    let fetch_limit = limit.saturating_add(1);
    let rows = sqlx::query_as::<_, (i64, Option<i64>, i64, String, String, String, i64)>(
        "SELECT MiningQueue.id, MiningQueue.rallyResultId, MiningArea.id, MiningArea.areaName, \
                Robot.robotName, User.username, \
                CAST(UNIX_TIMESTAMP(MiningQueue.miningEndTime) * 1000 AS SIGNED) \
         FROM MiningQueue \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE MiningQueue.playerNumber = 0 \
           AND MiningQueue.miningEndTime <= NOW() \
           AND (? IS NULL OR EXISTS (SELECT 1 \
                                      FROM MiningQueue UserQueue \
                                      INNER JOIN Robot UserRobot ON UserRobot.id = UserQueue.robotId \
                                      WHERE UserQueue.rallyResultId = MiningQueue.rallyResultId \
                                        AND UserRobot.userId = ?)) \
           AND (? IS NULL OR MiningArea.id = ?) \
         ORDER BY MiningQueue.miningEndTime DESC \
         LIMIT ?",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(mining_area_id)
    .bind(mining_area_id)
    .bind(fetch_limit)
    .fetch_all(pool)
    .await?;

    let has_more = rows.len() as i64 > limit;
    Ok((
        rows.into_iter()
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .map(
                |(
                    mining_queue_id,
                    rally_result_id,
                    mining_area_id,
                    mining_area_name,
                    robot_name,
                    username,
                    mining_end_time_millis,
                )| ActivityRecentRallyRecord {
                    mining_queue_id,
                    rally_result_id,
                    mining_area_id,
                    mining_area_name,
                    robot_name,
                    username,
                    mining_end_time_millis,
                },
            )
            .collect(),
        has_more,
    ))
}

pub async fn list_activity_rally_area_options(
    pool: &MySqlPool,
    maximum_areas: i64,
) -> Result<Vec<ActivityRallyAreaOption>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT MiningArea.id, MiningArea.areaName \
         FROM MiningQueue \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         WHERE MiningQueue.playerNumber = 0 \
           AND MiningQueue.miningEndTime <= NOW() \
         GROUP BY MiningArea.id, MiningArea.areaName \
         ORDER BY MiningArea.areaName \
         LIMIT ?",
    )
    .bind(maximum_areas)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(mining_area_id, area_name)| ActivityRallyAreaOption {
                mining_area_id,
                area_name,
            })
            .collect()
    })
}

pub async fn list_activity_recent_rally_participants(
    pool: &MySqlPool,
    maximum_rallies: i64,
) -> Result<Vec<ActivityRecentRallyParticipantRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32, String, String)>(
        "SELECT RecentQueue.id, MiningQueue.playerNumber, Robot.robotName, User.username \
         FROM (SELECT id, rallyResultId \
               FROM MiningQueue \
               WHERE playerNumber = 0 \
                 AND miningEndTime <= NOW() \
               ORDER BY miningEndTime DESC \
               LIMIT ?) RecentQueue \
         INNER JOIN MiningQueue ON MiningQueue.rallyResultId = RecentQueue.rallyResultId \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE MiningQueue.playerNumber > 0 \
         ORDER BY RecentQueue.id, MiningQueue.playerNumber",
    )
    .bind(maximum_rallies)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(mining_queue_id, player_number, robot_name, username)| {
                ActivityRecentRallyParticipantRecord {
                    mining_queue_id,
                    player_number,
                    robot_name,
                    username,
                }
            })
            .collect()
    })
}

pub async fn list_activity_rally_participants_for_queues(
    pool: &MySqlPool,
    mining_queue_ids: &[i64],
) -> Result<Vec<ActivityRecentRallyParticipantRecord>, sqlx::Error> {
    if mining_queue_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = std::iter::repeat_n("?", mining_queue_ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let query = format!(
        "SELECT RecentQueue.id, MiningQueue.playerNumber, Robot.robotName, User.username \
         FROM MiningQueue RecentQueue \
         INNER JOIN MiningQueue ON MiningQueue.rallyResultId = RecentQueue.rallyResultId \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE RecentQueue.id IN ({placeholders}) \
           AND MiningQueue.playerNumber > 0 \
         ORDER BY RecentQueue.id, MiningQueue.playerNumber"
    );
    let mut query = sqlx::query_as::<_, (i64, i32, String, String)>(&query);
    for mining_queue_id in mining_queue_ids {
        query = query.bind(mining_queue_id);
    }

    query
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(mining_queue_id, player_number, robot_name, username)| {
                    ActivityRecentRallyParticipantRecord {
                        mining_queue_id,
                        player_number,
                        robot_name,
                        username,
                    }
                })
                .collect()
        })
}

pub async fn rally_view_state(
    pool: &MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_user_result: bool,
) -> Result<Option<RallyViewStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, (String, String, String)>(
        "SELECT RallyResult.resultData, AiRobot.robotName, AiUser.username \
         FROM RallyResult \
         INNER JOIN MiningQueue ON MiningQueue.rallyResultId = RallyResult.id \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         INNER JOIN Robot AiRobot ON AiRobot.id = MiningArea.aiRobotId \
         INNER JOIN User AiUser ON AiUser.id = AiRobot.userId \
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
    .map(|row| {
        row.map(
            |(result_data, ai_robot_name, ai_username)| RallyViewStateRecord {
                result_data,
                ai_robot_name,
                ai_username,
            },
        )
    })
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
        sqlx::query_as::<_, (i32, i64, String, bool, f64, i32, i32, i32)>(
            "SELECT MiningQueue.playerNumber, MiningQueue.robotId, Robot.robotName, \
                    MiningQueue.claimed, COALESCE(MiningQueue.score, 0.0), \
                    CAST(COALESCE(SUM(MiningOreResult.amount), 0) AS SIGNED), \
                    CAST(COALESCE(SUM(COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED), \
                    CAST(COALESCE(SUM(MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED) \
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
    ) = if let Some((
        player_number,
        robot_id,
        robot_name,
        claimed,
        score,
        total_ore_mined,
        total_tax,
        total_reward,
    )) = viewer_context
    {
        (
            Some(player_number),
            Some(robot_id),
            Some(robot_name),
            Some(score),
            Some(total_ore_mined),
            Some(total_tax),
            Some(total_reward),
            claimed,
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
    sqlx::query_as::<_, (i32, String, String)>(
        "SELECT MiningQueue.playerNumber, Robot.robotName, User.username \
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
            .map(
                |(player_number, robot_name, username)| RallyViewParticipantRecord {
                    player_number,
                    robot_name,
                    username,
                },
            )
            .collect()
    })
}
