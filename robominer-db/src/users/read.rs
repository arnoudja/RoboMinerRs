use sqlx::MySqlPool;

use crate::UserRecord;

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    username: String,
    email: String,
    #[sqlx(rename = "password")]
    password_hash: String,
    #[sqlx(rename = "achievementPoints")]
    achievement_points: i32,
    #[sqlx(rename = "miningQueueSize")]
    mining_queue_size: i32,
    #[sqlx(rename = "sessionVersion")]
    session_version: i32,
}

impl From<UserRow> for UserRecord {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            achievement_points: row.achievement_points,
            mining_queue_size: row.mining_queue_size,
            session_version: row.session_version,
        }
    }
}

pub async fn get_user_by_id(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password, achievementPoints, miningQueueSize, sessionVersion \
         FROM User \
         WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(UserRecord::from))
}

/// Resolve a public username to a user id without loading credentials.
pub async fn get_user_id_by_username(
    pool: &MySqlPool,
    username: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM User WHERE username = ? LIMIT 1")
        .bind(username)
        .fetch_optional(pool)
        .await
}

/// Returns the current session version for a user, if the user exists.
pub async fn get_user_session_version(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar("SELECT sessionVersion FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}
