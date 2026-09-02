#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{
    ClaimAchievementStepRejection, ClaimAchievementStepRequest, claim_achievement_step,
    claim_user_results, cleanup_old_claimed_mining_queue_items_for_robot, list_user_depot_totals,
};
use robominer_test_support::{
    insert_area_supply, insert_claimed_mining_queue, insert_finished_queue, insert_mining_area,
    insert_ore, insert_ore_price, insert_ore_result_with_depot, insert_robot, insert_row_id,
    insert_user, insert_user_ore_asset, unique_prefix,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn user_depot_totals_survive_claimed_queue_cleanup_beyond_retention() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("depot-lifetime-retain");

    let ore_id = insert_ore(&pool, &format!("{prefix}-ore")).await;
    let ore_price_id = insert_ore_price(&pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(&pool, &prefix).await;
    let ai_robot_id =
        robominer_test_support::insert_ai_robot(&pool, &format!("{prefix}-ai"), "mine();", 1).await;
    let robot_id = insert_robot(&pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(&pool, &prefix, ore_price_id, ai_robot_id, 0).await;
    insert_area_supply(&pool, mining_area_id, ore_id, 10, 2).await;
    insert_user_ore_asset(&pool, user_id, ore_id, 0, 10_000).await;

    // Retention keeps 12 claimed queues; one extra run proves cleanup deletes history.
    const RUNS: i32 = 13;
    const DEPOT_PER_RUN: i32 = 7;
    let mut queue_ids = Vec::new();
    for offset in 0..RUNS {
        let queue_id =
            insert_finished_queue(&pool, mining_area_id, robot_id, -30 - offset, -20 - offset)
                .await;
        insert_ore_result_with_depot(&pool, queue_id, ore_id, 20, DEPOT_PER_RUN).await;
        queue_ids.push(queue_id);
    }

    let claimed = claim_user_results(&pool, user_id)
        .await
        .expect("claim should succeed");
    assert_eq!(claimed.claimed_queues, RUNS as u64);

    let before = list_user_depot_totals(&pool, user_id)
        .await
        .expect("depot totals before cleanup");
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].ore_id, ore_id);
    assert_eq!(before[0].amount, RUNS * DEPOT_PER_RUN);

    let summary = cleanup_old_claimed_mining_queue_items_for_robot(&pool, robot_id)
        .await
        .expect("cleanup should succeed");
    assert_eq!(summary.queues_deleted, 1);

    let remaining_queues: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE robotId = ? AND claimed = true")
            .bind(robot_id)
            .fetch_one(&pool)
            .await
            .expect("count remaining claimed queues");
    assert_eq!(remaining_queues, 12);

    let after = list_user_depot_totals(&pool, user_id)
        .await
        .expect("depot totals after cleanup");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].ore_id, ore_id);
    assert_eq!(
        after[0].amount,
        RUNS * DEPOT_PER_RUN,
        "lifetime depot totals must not shrink when claimed queue history is trimmed"
    );

    for queue_id in queue_ids {
        let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
    }
    robominer_test_support::cleanup_created_user(&pool, user_id).await;
    let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
        .bind(ore_id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[serial]
async fn claim_achievement_step_requires_depot_total_on_claimed_runs() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("depot-achievement-req");

    let ore_id = insert_ore(&pool, &format!("{prefix}-ore")).await;
    let ore_price_id = insert_ore_price(&pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(&pool, &prefix).await;
    let ai_robot_id =
        robominer_test_support::insert_ai_robot(&pool, &format!("{prefix}-ai"), "mine();", 1).await;
    let robot_id = insert_robot(&pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(&pool, &prefix, ore_price_id, ai_robot_id, 0).await;
    let achievement_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO Achievement (title, description) VALUES (?, ?)")
            .bind(format!("{prefix}-achievement"))
            .bind("depot requirement test"),
    )
    .await;

    sqlx::query(
        "INSERT INTO AchievementStep \
         (achievementId, step, achievementPoints, miningQueueReward, robotReward) \
         VALUES (?, 1, 5, 0, 0)",
    )
    .bind(achievement_id)
    .execute(&pool)
    .await
    .expect("failed to insert achievement step");

    sqlx::query(
        "INSERT INTO AchievementStepDepotTotalRequirement (achievementId, step, oreId, amount) \
         VALUES (?, 1, ?, 50)",
    )
    .bind(achievement_id)
    .bind(ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert depot requirement");

    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO UserAchievement (userId, achievementId, stepsClaimed) VALUES (?, ?, 0)",
        )
        .bind(user_id)
        .bind(achievement_id),
    )
    .await;

    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('{}')"),
    )
    .await;
    let mining_queue_id =
        insert_claimed_mining_queue(&pool, mining_area_id, robot_id, rally_result_id).await;
    insert_ore_result_with_depot(&pool, mining_queue_id, ore_id, 60, 30).await;
    sqlx::query(
        "INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax, depotAmount) \
         VALUES (?, ?, 60, 0, 30)",
    )
    .bind(robot_id)
    .bind(ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot lifetime depot total");

    let rejected = claim_achievement_step(
        &pool,
        ClaimAchievementStepRequest {
            user_id,
            achievement_id,
        },
    )
    .await
    .expect("claim should run")
    .expect_err("claim should fail below depot threshold");
    assert_eq!(rejected, ClaimAchievementStepRejection::RequirementsNotMet);

    sqlx::query("UPDATE RobotLifetimeResult SET depotAmount = 50 WHERE robotId = ? AND oreId = ?")
        .bind(robot_id)
        .bind(ore_id)
        .execute(&pool)
        .await
        .expect("failed to update depot amount");

    let claimed = claim_achievement_step(
        &pool,
        ClaimAchievementStepRequest {
            user_id,
            achievement_id,
        },
    )
    .await
    .expect("claim should run")
    .expect("claim should succeed at depot threshold");
    assert_eq!(claimed.step, 1);

    let steps_claimed: i32 = sqlx::query_scalar(
        "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = ?",
    )
    .bind(user_id)
    .bind(achievement_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load steps claimed");
    assert_eq!(steps_claimed, 1);

    let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
        .bind(mining_queue_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
        .bind(mining_queue_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
        .bind(rally_result_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM AchievementStepDepotTotalRequirement WHERE achievementId = ?")
        .bind(achievement_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM AchievementStep WHERE achievementId = ?")
        .bind(achievement_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Achievement WHERE id = ?")
        .bind(achievement_id)
        .execute(&pool)
        .await;
    robominer_test_support::cleanup_created_user(&pool, user_id).await;
    let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
        .bind(ore_id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[serial]
async fn list_achievement_page_depot_total_requirements_reports_progress() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("depot-achievement-page");

    let ore_id = insert_ore(&pool, &format!("{prefix}-ore")).await;
    let user_id = insert_user(&pool, &prefix).await;
    let achievement_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO Achievement (title, description) VALUES (?, ?)")
            .bind(format!("{prefix}-achievement"))
            .bind("depot page query test"),
    )
    .await;

    sqlx::query(
        "INSERT INTO AchievementStep \
         (achievementId, step, achievementPoints, miningQueueReward, robotReward) \
         VALUES (?, 1, 1, 0, 0)",
    )
    .bind(achievement_id)
    .execute(&pool)
    .await
    .expect("failed to insert achievement step");

    sqlx::query(
        "INSERT INTO AchievementStepDepotTotalRequirement (achievementId, step, oreId, amount) \
         VALUES (?, 1, ?, 40)",
    )
    .bind(achievement_id)
    .bind(ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert depot requirement");

    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO UserAchievement (userId, achievementId, stepsClaimed) VALUES (?, ?, 0)",
        )
        .bind(user_id)
        .bind(achievement_id),
    )
    .await;

    let requirements =
        robominer_db::list_achievement_page_depot_total_requirements_for_user(&pool, user_id)
            .await
            .expect("failed to load depot requirements");
    assert_eq!(requirements.len(), 1);
    assert_eq!(requirements[0].achievement_id, achievement_id);
    assert_eq!(requirements[0].ore_id, ore_id);
    assert_eq!(requirements[0].amount, 40);
    assert_eq!(requirements[0].current_amount, 0);

    let _ = sqlx::query("DELETE FROM AchievementStepDepotTotalRequirement WHERE achievementId = ?")
        .bind(achievement_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM AchievementStep WHERE achievementId = ?")
        .bind(achievement_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Achievement WHERE id = ?")
        .bind(achievement_id)
        .execute(&pool)
        .await;
    robominer_test_support::cleanup_created_user(&pool, user_id).await;
    let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
        .bind(ore_id)
        .execute(&pool)
        .await;
}
