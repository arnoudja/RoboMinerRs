#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_domain::{
    DomainError, load_mining_area_loadout, load_next_rally_loadout, load_pool_loadout,
};
use robominer_test_support::{PoolFixture, RallyFixture};
use serial_test::serial;
use sqlx::{Executor, MySqlPool};

async fn orphan_update(pool: &MySqlPool, sql: &'static str, bind_a: i64, bind_b: i64) {
    let mut conn = pool.acquire().await.expect("acquire connection");
    conn.execute("SET FOREIGN_KEY_CHECKS=0")
        .await
        .expect("disable foreign key checks");
    sqlx::query(sql)
        .bind(bind_a)
        .bind(bind_b)
        .execute(&mut *conn)
        .await
        .expect("orphan update");
    conn.execute("SET FOREIGN_KEY_CHECKS=1")
        .await
        .expect("re-enable foreign key checks");
}

#[tokio::test]
#[serial]
async fn missing_area_and_pool_return_none() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");

    assert!(
        load_mining_area_loadout(&pool, -1)
            .await
            .expect("query should succeed")
            .is_none()
    );
    assert!(
        load_pool_loadout(&pool, -1)
            .await
            .expect("query should succeed")
            .is_none()
    );
}

#[tokio::test]
#[serial]
async fn load_mining_area_reports_missing_ai_robot() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RallyFixture::create(&pool).await;
    let missing_robot_id = fixture.ai_robot_id + 9_000_000;

    orphan_update(
        &pool,
        "UPDATE MiningArea SET aiRobotId = ? WHERE id = ?",
        missing_robot_id,
        fixture.mining_area_id,
    )
    .await;

    let error = load_mining_area_loadout(&pool, fixture.mining_area_id)
        .await
        .expect_err("missing AI robot should fail integrity check");
    assert!(
        matches!(
            error,
            DomainError::ReferencedAiRobotMissing {
                mining_area_id,
                robot_id,
            } if mining_area_id == fixture.mining_area_id && robot_id == missing_robot_id
        ),
        "unexpected error: {error:?}"
    );

    orphan_update(
        &pool,
        "UPDATE MiningArea SET aiRobotId = ? WHERE id = ?",
        fixture.ai_robot_id,
        fixture.mining_area_id,
    )
    .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn load_next_rally_skips_orphaned_queue_robot() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RallyFixture::create(&pool).await;
    let missing_robot_id = fixture.queued_robot_id + 9_000_000;

    sqlx::query(
        "UPDATE MiningQueue SET creationTime = TIMESTAMPADD(SECOND, -3600, NOW()) WHERE id = ?",
    )
    .bind(fixture.mining_queue_id)
    .execute(&pool)
    .await
    .expect("age mining queue row");

    orphan_update(
        &pool,
        "UPDATE MiningQueue SET robotId = ? WHERE id = ?",
        missing_robot_id,
        fixture.mining_queue_id,
    )
    .await;

    // Queue listing joins Robot, so orphaned robot ids never reach the domain integrity check.
    assert!(
        load_next_rally_loadout(&pool, fixture.mining_area_id)
            .await
            .expect("query should succeed")
            .is_none()
    );

    orphan_update(
        &pool,
        "UPDATE MiningQueue SET robotId = ? WHERE id = ?",
        fixture.queued_robot_id,
        fixture.mining_queue_id,
    )
    .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn load_pool_reports_missing_mining_area() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = PoolFixture::create(&pool).await;
    let missing_area_id = fixture.mining_area_id + 9_000_000;

    orphan_update(
        &pool,
        "UPDATE Pool SET miningAreaId = ? WHERE id = ?",
        missing_area_id,
        fixture.pool_id,
    )
    .await;

    let error = load_pool_loadout(&pool, fixture.pool_id)
        .await
        .expect_err("missing mining area should fail integrity check");
    assert!(
        matches!(
            error,
            DomainError::ReferencedPoolMiningAreaMissing {
                pool_id,
                mining_area_id,
            } if pool_id == fixture.pool_id && mining_area_id == missing_area_id
        ),
        "unexpected error: {error:?}"
    );

    orphan_update(
        &pool,
        "UPDATE Pool SET miningAreaId = ? WHERE id = ?",
        fixture.mining_area_id,
        fixture.pool_id,
    )
    .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn load_pool_reports_missing_pool_robot() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = PoolFixture::create(&pool).await;
    let missing_robot_id = fixture.pool_robot_id + 9_000_000;

    orphan_update(
        &pool,
        "UPDATE PoolItem SET robotId = ? WHERE id = ?",
        missing_robot_id,
        fixture.pool_item_id,
    )
    .await;

    let error = load_pool_loadout(&pool, fixture.pool_id)
        .await
        .expect_err("missing pool robot should fail integrity check");
    assert!(
        matches!(
            error,
            DomainError::ReferencedPoolRobotMissing {
                pool_item_id,
                robot_id,
            } if pool_item_id == fixture.pool_item_id && robot_id == missing_robot_id
        ),
        "unexpected error: {error:?}"
    );

    orphan_update(
        &pool,
        "UPDATE PoolItem SET robotId = ? WHERE id = ?",
        fixture.pool_robot_id,
        fixture.pool_item_id,
    )
    .await;
    fixture.cleanup(&pool).await;
}
