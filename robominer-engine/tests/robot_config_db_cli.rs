mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn robot_config_states_report_pending_page_state() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRobotConfigFixture::create(&pool, true, true, 8).await;

    let update_output = run_engine(&fixture.update_args(&database_url));
    assert!(
        update_output.status.success(),
        "expected update-robot-config fixture setup to succeed\nstdout:\n{}\nstderr:\n{}",
        output_text(&update_output).0,
        output_text(&update_output).1
    );

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "robot-config-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected robot-config-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let program_source_prefix = format!("P\t{}\t", fixture.program_source_id);
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with(&program_source_prefix) && line.ends_with("\t8")),
        "expected program source state in stdout:\n{stdout}"
    );

    let robot_prefix = format!("R\t{}\t", fixture.robot_id);
    let robot_fields: Vec<&str> = stdout
        .lines()
        .find(|line| line.starts_with(&robot_prefix))
        .unwrap_or_else(|| panic!("expected robot config state in stdout:\n{stdout}"))
        .split('\t')
        .collect();
    assert_eq!(robot_fields.len(), 31);
    assert_eq!(robot_fields[2], "rust_bot");
    assert_eq!(robot_fields[3], fixture.program_source_id.to_string());
    assert_eq!(robot_fields[4], fixture.new_part_ids[0].to_string());
    assert!(robot_fields[5].ends_with("-new-0"));
    assert_eq!(robot_fields[10], fixture.new_part_ids[3].to_string());
    assert_eq!(robot_fields[16], fixture.new_part_ids[6].to_string());
    assert!(robot_fields[17].ends_with("-new-6"));
    assert_eq!(robot_fields[18], "14");
    assert_eq!(robot_fields[22], "56");
    assert_eq!(robot_fields[30], "true");

    let asset_prefix = format!(
        "A\t{}\t{}\t",
        fixture.robot_part_type_id, fixture.new_part_ids[3]
    );
    let asset_fields: Vec<&str> = stdout
        .lines()
        .find(|line| line.starts_with(&asset_prefix))
        .unwrap_or_else(|| panic!("expected robot part asset state in stdout:\n{stdout}"))
        .split('\t')
        .collect();
    assert_eq!(asset_fields.len(), 6);
    assert_eq!(asset_fields[4], "8");
    assert_eq!(asset_fields[5], "0");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_robot_config_updates_active_robot_when_not_queued() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRobotConfigFixture::create(&pool, false, true, 8).await;

    let output = run_engine(&fixture.update_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected update-robot-config to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Updated active configuration"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_active_updated(&pool).await;
    fixture.assert_no_pending_changes(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_robot_config_writes_pending_changes_when_queued() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRobotConfigFixture::create(&pool, true, true, 8).await;

    let output = run_engine(&fixture.update_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected update-robot-config to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Updated pending configuration"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_pending_updated(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_robot_config_updates_existing_pending_changes_when_queued() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRobotConfigFixture::create(&pool, true, true, 8).await;

    let first_output = run_engine(&fixture.update_args(&database_url));
    let (first_stdout, first_stderr) = output_text(&first_output);
    assert!(
        first_output.status.success(),
        "expected first update-robot-config to succeed\nstdout:\n{first_stdout}\nstderr:\n{first_stderr}"
    );
    assert!(
        first_stdout.contains("Updated pending configuration"),
        "unexpected stdout:\n{first_stdout}"
    );
    fixture.assert_pending_updated(&pool).await;

    let mut third_part_ids = [0_i64; 7];
    for index in 0..7 {
        third_part_ids[index] = insert_robot_config_part(
            &pool,
            fixture.robot_part_type_id,
            fixture.ore_id,
            fixture.ore_price_id,
            &format!("rust-robot-config-third-{index}-{}", fixture.robot_id),
            2,
        )
        .await;
        insert_user_robot_part_asset(&pool, fixture.user_id, third_part_ids[index], 1).await;
    }

    let second_output = run_engine(&fixture.update_args_for_parts(&database_url, &third_part_ids));
    let (second_stdout, second_stderr) = output_text(&second_output);
    assert!(
        second_output.status.success(),
        "expected second update-robot-config to succeed\nstdout:\n{second_stdout}\nstderr:\n{second_stderr}"
    );
    assert!(
        second_stdout.contains("Updated pending configuration"),
        "unexpected stdout:\n{second_stdout}"
    );
    assert!(
        second_stderr.is_empty(),
        "unexpected stderr:\n{second_stderr}"
    );

    fixture.assert_pending_parts(&pool, &third_part_ids).await;

    for robot_part_id in third_part_ids {
        let _ = sqlx::query("DELETE FROM RobotPart WHERE id = ?")
            .bind(robot_part_id)
            .execute(&pool)
            .await;
    }
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_robot_config_rejects_missing_unassigned_part() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRobotConfigFixture::create(&pool, false, false, 8).await;

    let output = run_engine(&fixture.update_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected update-robot-config to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("no unassigned robot part"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.assert_active_unchanged(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_robot_config_rejects_program_that_does_not_fit_memory() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRobotConfigFixture::create(&pool, false, true, 999).await;

    let output = run_engine(&fixture.update_args(&database_url));
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected update-robot-config to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("does not fit in memory"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.assert_active_unchanged(&pool).await;
    fixture.cleanup(&pool).await;
}
