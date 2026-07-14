mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn leaderboard_states_report_ranked_rows() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-leaderboard-cli");
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
    let user_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO User (username, email, password, achievementPoints) \
             VALUES (?, ?, 'test-password', 999999)",
        )
        .bind(format!("{prefix}-user"))
        .bind(format!("{prefix}@example.invalid")),
    )
    .await;
    let robot_id = insert_robot(&pool, user_id, &format!("{prefix}-robot"), "mine();").await;

    sqlx::query("UPDATE Robot SET totalMiningRuns = 4 WHERE id = ?")
        .bind(robot_id)
        .execute(&pool)
        .await
        .expect("failed to update robot total mining runs");
    sqlx::query(
        "INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax) \
         VALUES (?, ?, 400000, 0)",
    )
    .bind(robot_id)
    .bind(ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot lifetime result");

    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
        )
        .bind(format!("{prefix}-area"))
        .bind(ore_price_id)
        .bind(robot_id),
    )
    .await;
    sqlx::query(
        "INSERT INTO RobotMiningAreaScore (robotId, miningAreaId, totalRuns, score) \
         VALUES (?, ?, 7, 98765.5)",
    )
    .bind(robot_id)
    .bind(mining_area_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot mining area score");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "leaderboard-states".to_string(),
        "--max-entries".to_string(),
        "100".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected leaderboard-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
    assert!(
        stdout.contains(&format!("A\t{mining_area_id}\t{prefix}-area")),
        "expected mining area row in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "S\t{mining_area_id}\t{prefix}-robot\t{prefix}-user\t98765.5\t7"
        )),
        "expected mining area score row in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("R\t{prefix}-robot\t{prefix}-user\t100000")),
        "expected top robot row in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("U\t{prefix}-user\t999999")),
        "expected top user row in stdout:\n{stdout}"
    );

    let _ = sqlx::query("DELETE FROM RobotMiningAreaScore WHERE robotId = ?")
        .bind(robot_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM RobotLifetimeResult WHERE robotId = ?")
        .bind(robot_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
        .bind(robot_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
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
}
