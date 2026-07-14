mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn claim_achievement_step_applies_rewards() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestAchievementFixture::create(&pool, false).await;

    let output = run_engine(&fixture.claim_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected claim-achievement-step to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Claimed achievement"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_claimed_rewards(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn claim_achievement_step_unlocks_successor() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestAchievementFixture::create(&pool, false).await;
    fixture.add_successor(&pool).await;

    let output = run_engine(&fixture.claim_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected claim-achievement-step to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_successor_unlocked(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn reconcile_successor_unlocks_after_late_predecessor_link() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestAchievementFixture::create(&pool, false).await;

    let output = run_engine(&fixture.claim_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected claim-achievement-step to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_claimed_rewards(&pool).await;
    fixture.add_successor(&pool).await;
    fixture.assert_successor_not_unlocked(&pool).await;

    robominer_db::list_achievement_claim_states_for_user(&pool, fixture.user_id)
        .await
        .expect("failed to reconcile successor unlocks");

    fixture.assert_successor_unlocked(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn claim_achievement_step_adds_robot_reward() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;
    let fixture = TestAchievementFixture::create(&pool, true).await;

    let output = run_engine(&fixture.claim_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected claim-achievement-step to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_robot_reward(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn claim_achievement_step_rejects_unmet_requirements() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestAchievementFixture::create_unmet(&pool).await;

    let output = run_engine(&fixture.claim_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected claim-achievement-step to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("requirements are not met"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.assert_not_claimed(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn achievement_states_report_claimable_and_progress() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestAchievementFixture::create_unmet(&pool).await;
    let robot_id = insert_robot(&pool, fixture.user_id, "achievement-state-robot", "mine();").await;

    sqlx::query(
        "INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax) VALUES (?, ?, 12, 0)",
    )
    .bind(robot_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot lifetime result");

    let ore_price_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)").bind(format!(
            "achievement-state-price-{}",
            fixture.achievement_id
        )),
    )
    .await;
    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
        )
        .bind(format!("achievement-state-area-{}", fixture.achievement_id))
        .bind(ore_price_id)
        .bind(robot_id),
    )
    .await;
    sqlx::query(
        "INSERT INTO RobotMiningAreaScore (robotId, miningAreaId, totalRuns, score) \
         VALUES (?, ?, 3, 42.5)",
    )
    .bind(robot_id)
    .bind(mining_area_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot mining area score");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "achievement-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected achievement-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
    assert!(
        stdout.contains(&format!("C\t{}\ttrue", fixture.achievement_id)),
        "expected claimable achievement state in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("T\t{}\t12", fixture.ore_id)),
        "expected ore progress state in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("S\t{mining_area_id}\t42.5")),
        "expected mining area score state in stdout:\n{stdout}"
    );

    sqlx::query("DELETE FROM RobotMiningAreaScore WHERE robotId = ?")
        .bind(robot_id)
        .execute(&pool)
        .await
        .expect("failed to delete robot mining area score");
    sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
        .execute(&pool)
        .await
        .expect("failed to delete mining area");
    sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(ore_price_id)
        .execute(&pool)
        .await
        .expect("failed to delete ore price");
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn achievement_page_states_report_display_model() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestAchievementFixture::create_unmet(&pool).await;
    let robot_id = insert_robot(
        &pool,
        fixture.user_id,
        "achievement-page-state-robot",
        "mine();",
    )
    .await;

    sqlx::query(
        "INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax) VALUES (?, ?, 12, 0)",
    )
    .bind(robot_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot lifetime result");

    let ore_price_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)").bind(format!(
            "achievement-page-state-price-{}",
            fixture.achievement_id
        )),
    )
    .await;
    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
        )
        .bind(format!(
            "achievement-page-state-area-{}",
            fixture.achievement_id
        ))
        .bind(ore_price_id)
        .bind(robot_id),
    )
    .await;
    sqlx::query(
        "INSERT INTO RobotMiningAreaScore (robotId, miningAreaId, totalRuns, score) \
         VALUES (?, ?, 3, 42.5)",
    )
    .bind(robot_id)
    .bind(mining_area_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot mining area score");
    sqlx::query(
        "INSERT INTO AchievementStepMiningScoreRequirement \
         (achievementId, step, miningAreaId, minimumScore) VALUES (?, ?, ?, 40.0)",
    )
    .bind(fixture.achievement_id)
    .bind(fixture.achievement_step)
    .bind(mining_area_id)
    .execute(&pool)
    .await
    .expect("failed to insert achievement score requirement");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "achievement-page-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected achievement-page-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let view_line = find_prefixed_line(&stdout, "V\t");
    assert_eq!(view_line, vec!["V", "1"]);

    let achievement_line = find_prefixed_line(&stdout, &format!("A\t{}\t", fixture.achievement_id));
    assert_eq!(achievement_line.len(), 19);
    assert!(achievement_line[2].ends_with("-achievement"));
    assert_eq!(achievement_line[3], "test achievement");
    assert_eq!(achievement_line[4], "0");
    assert_eq!(achievement_line[5], "1");
    assert_eq!(achievement_line[6], "0");
    assert_eq!(achievement_line[7], "7");
    assert_eq!(achievement_line[8], fixture.achievement_step.to_string());
    assert_eq!(achievement_line[9], "7");
    assert_eq!(achievement_line[10], "2");
    assert_eq!(achievement_line[11], "0");
    assert_eq!(achievement_line[12], fixture.ore_id.to_string());
    assert!(achievement_line[13].ends_with("-ore"));
    assert_eq!(achievement_line[14], "5");
    assert_eq!(achievement_line[15], "80");
    assert_eq!(achievement_line[16], "");
    assert_eq!(achievement_line[17], "");
    assert_eq!(achievement_line[18], "true");

    let total_line = find_prefixed_line(&stdout, &format!("T\t{}\t", fixture.achievement_id));
    assert_eq!(total_line.len(), 6);
    assert_eq!(total_line[2], fixture.ore_id.to_string());
    assert!(total_line[3].ends_with("-ore"));
    assert_eq!(total_line[4], "10");
    assert_eq!(total_line[5], "12");

    let score_line = find_prefixed_line(&stdout, &format!("S\t{}\t", fixture.achievement_id));
    assert_eq!(score_line.len(), 6);
    assert_eq!(score_line[2], mining_area_id.to_string());
    assert!(score_line[3].starts_with("achievement-page-state-area-"));
    assert_eq!(score_line[4], "40");
    assert_eq!(score_line[5], "42.5");

    sqlx::query(
        "DELETE FROM AchievementStepMiningScoreRequirement \
         WHERE achievementId = ? AND step = ?",
    )
    .bind(fixture.achievement_id)
    .bind(fixture.achievement_step)
        .execute(&pool)
        .await
        .expect("failed to delete achievement score requirement");
    sqlx::query("DELETE FROM RobotMiningAreaScore WHERE robotId = ?")
        .bind(robot_id)
        .execute(&pool)
        .await
        .expect("failed to delete robot mining area score");
    sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
        .execute(&pool)
        .await
        .expect("failed to delete mining area");
    sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(ore_price_id)
        .execute(&pool)
        .await
        .expect("failed to delete ore price");
    fixture.cleanup(&pool).await;
}

