#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn claim_next_mining_rally_leases_at_most_rally_size() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-claim-truncate");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await;
    let ore_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
    )
    .await;
    let ore_price_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
            .bind(format!("{prefix}-price")),
    )
    .await;
    let ai_robot_id = insert_ai_robot(&pool, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 60, 0, ?)",
        )
        .bind(format!("{prefix}-area"))
        .bind(ore_price_id)
        .bind(ai_robot_id),
    )
    .await;
    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningAreaOreSupply (miningAreaId, oreId, supply, radius) \
             VALUES (?, ?, 10, 2)",
        )
        .bind(mining_area_id)
        .bind(ore_id),
    )
    .await;

    let mut queue_ids = Vec::new();
    let mut robot_ids = Vec::new();
    let mut user_ids = Vec::new();
    for index in 0..6 {
        let owner_id = insert_test_user(
            &pool,
            &format!("{prefix}-user-{index}"),
            &format!("{prefix}-{index}@example.invalid"),
            "test-password",
        )
        .await;
        user_ids.push(owner_id);
        let robot_id = insert_robot(
            &pool,
            owner_id,
            &format!("{prefix}-robot-{index}"),
            "mine();",
        )
        .await;
        robot_ids.push(robot_id);
        let queue_id = insert_row_id(
            &pool,
            sqlx::query(
                "INSERT INTO MiningQueue (miningAreaId, robotId, creationTime, miningEndTime) \
                 VALUES (?, ?, TIMESTAMPADD(SECOND, -3600, NOW()), NULL)",
            )
            .bind(mining_area_id)
            .bind(robot_id),
        )
        .await;
        queue_ids.push(queue_id);
    }

    let claimed =
        robominer_db::claim_next_mining_rally_queue_for_area(&pool, mining_area_id, 4, 10)
            .await
            .expect("claim should succeed")
            .expect("rally should be ready");
    assert_eq!(claimed.len(), 4, "claim must lease at most rally_size rows");

    let leased: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM MiningQueue \
         WHERE miningAreaId = ? \
           AND processingLeaseUntil IS NOT NULL \
           AND processingLeaseUntil > NOW()",
    )
    .bind(mining_area_id)
    .fetch_one(&pool)
    .await
    .expect("count leased rows");
    assert_eq!(leased, 4);

    let remaining = robominer_db::list_next_mining_rally_queue_for_area(&pool, mining_area_id)
        .await
        .expect("list remaining free queue heads");
    assert_eq!(
        remaining.len(),
        2,
        "unclaimed free heads should remain claimable"
    );

    for queue_id in queue_ids {
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId = ?")
        .bind(mining_area_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM AIRobot WHERE id = ?")
        .bind(ai_robot_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(ore_price_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
        .bind(ore_id)
        .execute(&pool)
        .await;
    for robot_id in robot_ids {
        let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
            .bind(robot_id)
            .execute(&pool)
            .await;
    }
    for owner_id in user_ids {
        cleanup_created_user(&pool, owner_id).await;
    }
    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn list_next_claim_rally_candidates_reports_busy_seconds() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;

    sqlx::query(
        "UPDATE Robot SET miningEndTime = TIMESTAMPADD(SECOND, 25, NOW()), \
             rechargeEndTime = TIMESTAMPADD(SECOND, 40, NOW()) \
         WHERE id = ?",
    )
    .bind(fixture.queued_robot_id)
    .execute(&pool)
    .await
    .expect("set robot busy");

    let candidates = robominer_db::list_next_claim_rally_candidates(&pool)
        .await
        .expect("list candidates");
    let match_candidate = candidates
        .iter()
        .find(|c| c.mining_area_id == fixture.mining_area_id)
        .expect("fixture area should appear in candidates");
    assert!(
        match_candidate.busy_seconds >= 25,
        "busy_seconds should reflect mining/recharge end: {match_candidate:?}"
    );

    let area_candidates: Vec<_> = candidates
        .iter()
        .filter(|c| c.mining_area_id == fixture.mining_area_id)
        .cloned()
        .collect();
    let delay = robominer_domain::next_claimable_rally_delay_seconds(&area_candidates);
    assert!(
        delay.is_some_and(|d| d >= 25),
        "partial busy queue should wait for free/expiry, got {delay:?}"
    );

    fixture.cleanup(&pool).await;
}
