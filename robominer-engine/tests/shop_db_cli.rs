mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn buy_robot_part_deducts_ore_and_adds_owned_part() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestShopFixture::create(&pool, 25, 10, 0, false).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "buy-robot-part".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--robot-part-id".to_string(),
        fixture.robot_part_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected buy-robot-part to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Bought robot part"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_ore_amount(&pool, 15).await;
    fixture.assert_robot_part_total_owned(&pool, Some(1)).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn buy_robot_part_insufficient_funds_rolls_back() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestShopFixture::create(&pool, 3, 10, 0, false).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "buy-robot-part".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--robot-part-id".to_string(),
        fixture.robot_part_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected buy-robot-part to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("insufficient funds"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.assert_ore_amount(&pool, 3).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn sell_robot_part_refunds_half_cost_and_deletes_zero_owned_asset() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestShopFixture::create(&pool, 0, 10, 1, false).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "sell-robot-part".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--robot-part-id".to_string(),
        fixture.robot_part_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected sell-robot-part to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Sold robot part"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_ore_amount(&pool, 5).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn sell_robot_part_rejects_part_currently_used_by_robot() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestShopFixture::create(&pool, 0, 10, 1, true).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "sell-robot-part".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--robot-part-id".to_string(),
        fixture.robot_part_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected sell-robot-part to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("no unassigned robot part"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.assert_ore_amount(&pool, 0).await;
    fixture.assert_robot_part_total_owned(&pool, Some(1)).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn shop_robot_part_states_report_owned_and_assigned_counts() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestShopFixture::create(&pool, 25, 10, 1, true).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "shop-robot-part-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected shop-robot-part-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let prefix = format!("{}\t", fixture.robot_part_id);
    let state_line = stdout
        .lines()
        .find(|line| line.starts_with(&prefix))
        .expect("expected shop state for fixture robot part");
    assert_eq!(
        state_line,
        format!("{}\t1\t1\t0\tfalse\tfalse", fixture.robot_part_id)
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn shop_catalog_states_report_catalog_rows() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestShopFixture::create(&pool, 25, 10, 0, false).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "shop-catalog-states".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected shop-catalog-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let ore_prefix = format!("O\t{}\t", fixture.ore_id);
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with(&ore_prefix) && line.ends_with("-ore")),
        "expected ore row in stdout:\n{stdout}"
    );

    let type_prefix = format!("T\t{}\t", fixture.robot_part_type_id);
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with(&type_prefix) && line.ends_with("-type")),
        "expected robot part type row in stdout:\n{stdout}"
    );

    let part_prefix = format!("P\t{}\t", fixture.robot_part_id);
    let part_fields: Vec<&str> = stdout
        .lines()
        .find(|line| line.starts_with(&part_prefix))
        .unwrap_or_else(|| panic!("expected robot part row in stdout:\n{stdout}"))
        .split('\t')
        .collect();
    assert_eq!(part_fields.len(), 18);
    assert_eq!(part_fields[2], fixture.robot_part_type_id.to_string());
    assert_eq!(part_fields[3], fixture.ore_id.to_string());
    assert!(part_fields[4].ends_with("-ore"));
    assert!(part_fields[5].ends_with("-part"));
    assert_eq!(part_fields[15], "1");
    assert_eq!(part_fields[16], "1");
    assert_eq!(part_fields[17], "1");

    let cost_prefix = format!("C\t{}\t{}\t", fixture.robot_part_id, fixture.ore_id);
    let cost_fields: Vec<&str> = stdout
        .lines()
        .find(|line| line.starts_with(&cost_prefix))
        .unwrap_or_else(|| panic!("expected robot part cost row in stdout:\n{stdout}"))
        .split('\t')
        .collect();
    assert_eq!(cost_fields.len(), 5);
    assert!(cost_fields[3].ends_with("-ore"));
    assert_eq!(cost_fields[4], "10");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn user_ore_asset_states_report_user_balances() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestShopFixture::create(&pool, 25, 10, 0, false).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "user-ore-asset-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected user-ore-asset-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let summary_fields = find_prefixed_line(&stdout, "U\t");
    assert_eq!(summary_fields.len(), 5);
    assert!(summary_fields[1].ends_with("-user"));
    assert_eq!(summary_fields[2], "0");
    assert_eq!(summary_fields[3], "0");
    assert_eq!(summary_fields[4], "1");

    let state_fields = find_prefixed_line(&stdout, &format!("O\t{}\t", fixture.ore_id));
    assert_eq!(state_fields.len(), 5);
    assert_eq!(state_fields[1], fixture.ore_id.to_string());
    assert!(state_fields[2].ends_with("-ore"));
    assert_eq!(state_fields[3], "25");
    assert_eq!(state_fields[4], "100");

    fixture.cleanup(&pool).await;
}

