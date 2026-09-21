#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{list_leaderboard_top_users, load_leaderboard_viewer_standing};
use robominer_test_support::{insert_user, unique_prefix};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn list_leaderboard_top_users_includes_user_id_1() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let created = ensure_user_id_1(&pool).await;
    let original_points: i32 =
        sqlx::query_scalar("SELECT achievementPoints FROM User WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("user 1 points should load");
    let username: String = sqlx::query_scalar("SELECT username FROM User WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("user 1 username should load");

    sqlx::query("UPDATE User SET achievementPoints = 8888888 WHERE id = 1")
        .execute(&pool)
        .await
        .expect("failed to bump user 1 achievement points");

    let top_users = list_leaderboard_top_users(&pool, 10)
        .await
        .expect("top users should load");

    sqlx::query("UPDATE User SET achievementPoints = ? WHERE id = 1")
        .bind(original_points)
        .execute(&pool)
        .await
        .expect("failed to restore user 1 achievement points");
    cleanup_created_user_id_1(&pool, created).await;

    assert!(
        top_users
            .iter()
            .any(|user| user.username == username && user.achievement_points == 8_888_888),
        "user id 1 should appear on the top players list"
    );
}

#[tokio::test]
#[serial]
async fn load_leaderboard_viewer_standing_counts_user_id_1_in_rank() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let created = ensure_user_id_1(&pool).await;
    let original_points: i32 =
        sqlx::query_scalar("SELECT achievementPoints FROM User WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("user 1 points should load");
    let prefix = unique_prefix("rust-db-leaderboard-rank");
    let other_user_id = insert_user(&pool, &prefix).await;

    sqlx::query("UPDATE User SET achievementPoints = 7777777 WHERE id = 1")
        .execute(&pool)
        .await
        .expect("failed to bump user 1 achievement points");
    sqlx::query("UPDATE User SET achievementPoints = 0 WHERE id = ?")
        .bind(other_user_id)
        .execute(&pool)
        .await
        .expect("failed to set other user achievement points");

    let standing = load_leaderboard_viewer_standing(&pool, other_user_id)
        .await
        .expect("viewer standing should load");
    let expected_rank: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) + 1 \
         FROM User \
         WHERE achievementPoints > 0 \
            OR (achievementPoints = 0 AND id < ?)",
    )
    .bind(other_user_id)
    .fetch_one(&pool)
    .await
    .expect("expected rank should load");

    sqlx::query("UPDATE User SET achievementPoints = ? WHERE id = 1")
        .bind(original_points)
        .execute(&pool)
        .await
        .expect("failed to restore user 1 achievement points");
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(other_user_id)
        .execute(&pool)
        .await;
    cleanup_created_user_id_1(&pool, created).await;

    assert_eq!(standing.achievement_points, 0);
    assert_eq!(
        standing.achievement_rank, expected_rank,
        "achievement rank should count every user, including id 1"
    );
}

async fn ensure_user_id_1(pool: &robominer_db::MySqlPool) -> bool {
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM User WHERE id = 1")
        .fetch_optional(pool)
        .await
        .expect("user 1 lookup should succeed");
    if exists.is_some() {
        return false;
    }

    let prefix = unique_prefix("rust-db-user-id-1");
    sqlx::query(
        "INSERT INTO User (id, username, email, password) VALUES (1, ?, ?, 'test-password-1')",
    )
    .bind(format!("{prefix}-user"))
    .bind(format!("{prefix}@example.invalid"))
    .execute(pool)
    .await
    .expect("failed to insert user id 1");
    true
}

async fn cleanup_created_user_id_1(pool: &robominer_db::MySqlPool, created: bool) {
    if created {
        let _ = sqlx::query("DELETE FROM User WHERE id = 1")
            .execute(pool)
            .await;
    }
}
