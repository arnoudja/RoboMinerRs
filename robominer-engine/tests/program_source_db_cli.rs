mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn create_program_source_inserts_and_verifies_source() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "create-program-source".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--source-name".to_string(),
        "new source".to_string(),
        "--source-code".to_string(),
        "move(1);".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected create-program-source to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Created program source"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let created_id = parse_created_program_source_id(&stdout);
    let (source_name, source_code, verified): (String, String, bool) =
        sqlx::query_as("SELECT sourceName, sourceCode, verified FROM ProgramSource WHERE id = ?")
            .bind(created_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load created program source");
    assert_eq!(source_name, "new source");
    assert_eq!(source_code, "move(1);");
    assert!(verified);

    fixture
        .cleanup_extra_program_source(&pool, created_id)
        .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn program_source_states_report_editor_rows() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;
    fixture.create_linked_robots(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "program-source-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected program-source-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let source_prefix = format!("S\t{}\t", fixture.program_source_id);
    let fields: Vec<&str> = stdout
        .lines()
        .find(|line| line.starts_with(&source_prefix))
        .unwrap_or_else(|| panic!("expected program source state in stdout:\n{stdout}"))
        .split('\t')
        .collect();
    assert_eq!(fields.len(), 9);
    assert!(fields[2].ends_with("-source"));
    assert_eq!(fields[3], "move(1);");
    assert_eq!(fields[4], "true");
    assert_eq!(fields[5], "1");
    assert_eq!(fields[6], "");
    assert_eq!(fields[7], "3");
    assert_eq!(fields[8], fixture.user_id.to_string());

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_program_source_applies_to_idle_robots_and_reports_warnings() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;
    fixture.create_linked_robots(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "update-program-source".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--program-source-id".to_string(),
        fixture.program_source_id.to_string(),
        "--source-name".to_string(),
        "updated source".to_string(),
        "--source-code".to_string(),
        "move(2);".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected update-program-source to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("applied to 2 robot(s)")
            && stdout.contains("WARNING Unable to apply the code to robot")
            && stdout.contains("Not enough memory."),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_source_updated_and_applied(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn apply_program_source_to_linked_robots_updates_idle_robots_and_warns() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;
    fixture.create_linked_robots(&pool).await;

    let applied = robominer_domain::apply_program_source_to_linked_robots(
        &pool,
        fixture.user_id,
        fixture.program_source_id,
    )
    .await
    .expect("failed to apply program source to linked robots");

    assert_eq!(applied.applied_robots, 2);
    assert_eq!(applied.warnings.len(), 1);
    assert!(
        applied.warnings.iter().any(|warning| {
            warning.reason == robominer_db::ProgramSourceApplyWarningReason::NotEnoughMemory
        })
    );

    let robot_ids = fixture.robot_ids.borrow().clone();
    let busy_pending_source: String = sqlx::query_scalar(
        "SELECT sourceCode FROM PendingRobotChanges WHERE robotId = ?",
    )
    .bind(robot_ids[1])
    .fetch_one(&pool)
    .await
    .expect("busy robot should have pending source update");

    assert_eq!(busy_pending_source, "move(1);");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn apply_program_source_to_busy_robot_with_pending_updates_source_only() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;
    fixture.create_linked_robots(&pool).await;

    let robot_ids = fixture.robot_ids.borrow().clone();
    let busy_robot_id = robot_ids[1];

    sqlx::query(
        "INSERT INTO PendingRobotChanges \
         (robotId, sourceCode, oreContainerId, miningUnitId, batteryId, memoryModuleId, \
          cpuId, engineId, oreScannerId, oldOreContainerId, oldMiningUnitId, oldBatteryId, \
          oldMemoryModuleId, oldCpuId, oldEngineId, oldOreScannerId, rechargeTime, maxOre, \
          miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
          robotSize, scanTime, scanDistance) \
         SELECT id, 'move(0);', oreContainerId, miningUnitId, batteryId, memoryModuleId, cpuId, \
                engineId, oreScannerId, oreContainerId, miningUnitId, batteryId, memoryModuleId, \
                cpuId, engineId, oreScannerId, rechargeTime, maxOre, miningSpeed, maxTurns, \
                memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, robotSize, scanTime, \
                scanDistance \
         FROM Robot WHERE id = ?",
    )
    .bind(busy_robot_id)
    .execute(&pool)
    .await
    .expect("failed to seed pending robot changes");

    let applied = robominer_domain::apply_program_source_to_linked_robots(
        &pool,
        fixture.user_id,
        fixture.program_source_id,
    )
    .await
    .expect("failed to apply program source to linked robots");

    assert_eq!(applied.applied_robots, 2);
    assert_eq!(applied.warnings.len(), 1);

    let (pending_source, pending_ore_container): (String, i64) = sqlx::query_as(
        "SELECT sourceCode, oreContainerId FROM PendingRobotChanges WHERE robotId = ?",
    )
    .bind(busy_robot_id)
    .fetch_one(&pool)
    .await
    .expect("busy robot should still have pending changes");

    assert_eq!(pending_source, "move(1);");
    let active_ore_container: i64 =
        sqlx::query_scalar("SELECT oreContainerId FROM Robot WHERE id = ?")
            .bind(busy_robot_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load active ore container");
    assert_eq!(pending_ore_container, active_ore_container);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn delete_program_source_removes_unlinked_source() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "delete-program-source".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--program-source-id".to_string(),
        fixture.program_source_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected delete-program-source to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Deleted program source"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_source_deleted(&pool).await;
    fixture.cleanup_without_source(&pool).await;
}

#[tokio::test]
#[serial]
async fn delete_program_source_rejects_source_linked_to_robot() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;
    fixture.create_linked_robots(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "delete-program-source".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--program-source-id".to_string(),
        fixture.program_source_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected delete-program-source to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("still linked to a robot"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_program_source_rejects_wrong_owner() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestProgramSourceFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "update-program-source".to_string(),
        "--user-id".to_string(),
        fixture.other_user_id.to_string(),
        "--program-source-id".to_string(),
        fixture.program_source_id.to_string(),
        "--source-name".to_string(),
        "wrong owner".to_string(),
        "--source-code".to_string(),
        "move(2);".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected update-program-source to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("unknown program source"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.assert_source_unchanged(&pool).await;
    fixture.cleanup(&pool).await;
}

