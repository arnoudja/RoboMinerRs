use crate::assert_sql_safe;
use sqlx::MySqlPool;

use crate::{
    ActivityRallyAreaOption, ActivityRecentRallyParticipantRecord, ActivityRecentRallyRecord,
    ActivityRecentUserRecord,
};

#[derive(sqlx::FromRow)]
struct ActivityRecentUserRow {
    #[sqlx(rename = "id")]
    user_id: i64,
    username: String,
    #[sqlx(rename = "lastLoginTimeMillis")]
    last_login_time_millis: i64,
}

impl From<ActivityRecentUserRow> for ActivityRecentUserRecord {
    fn from(row: ActivityRecentUserRow) -> Self {
        Self {
            user_id: row.user_id,
            username: row.username,
            last_login_time_millis: row.last_login_time_millis,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ActivityRecentRallyRow {
    #[sqlx(rename = "id")]
    mining_queue_id: i64,
    #[sqlx(rename = "rallyResultId")]
    rally_result_id: Option<i64>,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "areaName")]
    mining_area_name: String,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    username: String,
    #[sqlx(rename = "miningEndTimeMillis")]
    mining_end_time_millis: i64,
}

impl From<ActivityRecentRallyRow> for ActivityRecentRallyRecord {
    fn from(row: ActivityRecentRallyRow) -> Self {
        Self {
            mining_queue_id: row.mining_queue_id,
            rally_result_id: row.rally_result_id,
            mining_area_id: row.mining_area_id,
            mining_area_name: row.mining_area_name,
            robot_name: row.robot_name,
            username: row.username,
            mining_end_time_millis: row.mining_end_time_millis,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ActivityRecentRallyParticipantRow {
    #[sqlx(rename = "id")]
    mining_queue_id: i64,
    #[sqlx(rename = "playerNumber")]
    player_number: i32,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    username: String,
}

impl From<ActivityRecentRallyParticipantRow> for ActivityRecentRallyParticipantRecord {
    fn from(row: ActivityRecentRallyParticipantRow) -> Self {
        Self {
            mining_queue_id: row.mining_queue_id,
            player_number: row.player_number,
            robot_name: row.robot_name,
            username: row.username,
        }
    }
}

pub async fn list_activity_recent_users(
    pool: &MySqlPool,
    maximum_users: i64,
) -> Result<Vec<ActivityRecentUserRecord>, sqlx::Error> {
    sqlx::query_as::<_, ActivityRecentUserRow>(
        "SELECT id, username, \
                CAST(UNIX_TIMESTAMP(lastLoginTime) * 1000 AS SIGNED) AS lastLoginTimeMillis \
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
            .map(ActivityRecentUserRecord::from)
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
    let rows = sqlx::query_as::<_, ActivityRecentRallyRow>(
        "SELECT MiningQueue.id, MiningQueue.rallyResultId, MiningArea.id AS miningAreaId, \
                MiningArea.areaName, Robot.robotName, User.username, \
                CAST(UNIX_TIMESTAMP(MiningQueue.miningEndTime) * 1000 AS SIGNED) \
                  AS miningEndTimeMillis \
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
            .map(ActivityRecentRallyRecord::from)
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
    sqlx::query_as::<_, ActivityRecentRallyParticipantRow>(
        "SELECT RecentQueue.id AS id, MiningQueue.playerNumber AS playerNumber, \
                Robot.robotName AS robotName, User.username AS username \
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
            .map(ActivityRecentRallyParticipantRecord::from)
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

    let placeholders = crate::in_placeholders(mining_queue_ids.len());
    let query = format!(
        "SELECT RecentQueue.id AS id, MiningQueue.playerNumber AS playerNumber, \
                Robot.robotName AS robotName, User.username AS username \
         FROM MiningQueue RecentQueue \
         INNER JOIN MiningQueue ON MiningQueue.rallyResultId = RecentQueue.rallyResultId \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE RecentQueue.id IN ({placeholders}) \
           AND MiningQueue.playerNumber > 0 \
         ORDER BY RecentQueue.id, MiningQueue.playerNumber"
    );
    let mut query = sqlx::query_as::<_, ActivityRecentRallyParticipantRow>(assert_sql_safe(query));
    for mining_queue_id in mining_queue_ids {
        query = query.bind(mining_queue_id);
    }

    query.fetch_all(pool).await.map(|rows| {
        rows.into_iter()
            .map(ActivityRecentRallyParticipantRecord::from)
            .collect()
    })
}
