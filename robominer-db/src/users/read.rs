use sqlx::MySqlPool;

use crate::UserRecord;

pub async fn get_user_by_id(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, String, String, i32, i32, i32)>(
        "SELECT id, username, email, password, achievementPoints, miningQueueSize, sessionVersion \
         FROM User \
         WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(
                id,
                username,
                email,
                password_hash,
                achievement_points,
                mining_queue_size,
                session_version,
            )| {
                UserRecord {
                    id,
                    username,
                    email,
                    password_hash,
                    achievement_points,
                    mining_queue_size,
                    session_version,
                }
            },
        )
    })
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

/// Count users still storing legacy `sha256:` password hashes (upgraded on login).
pub async fn count_legacy_password_hashes(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM User WHERE password LIKE 'sha256:%'")
        .fetch_one(pool)
        .await?;
    Ok(count.max(0) as u64)
}
