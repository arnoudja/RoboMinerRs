use robominer_db::{
    UpdateRobotConfigRequest, list_robot_config_part_asset_states, list_robot_config_states,
    list_robot_lifetime_ore_stats, list_robot_mining_area_stats, load_robot_stats_header,
    update_robot_config,
};
use robominer_test_support::RobotConfigFixture;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn list_robot_config_states_reflects_pending_loadout_after_update() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db robots test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RobotConfigFixture::create(&pool, true, true, 8).await;

    let updated = update_robot_config(
        &pool,
        UpdateRobotConfigRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            robot_name: "rust_bot".to_string(),
            program_source_id: fixture.program_source_id,
            ore_container_id: fixture.new_part_ids[0],
            mining_unit_id: fixture.new_part_ids[1],
            battery_id: fixture.new_part_ids[2],
            memory_module_id: fixture.new_part_ids[3],
            cpu_id: fixture.new_part_ids[4],
            engine_id: fixture.new_part_ids[5],
            ore_scanner_id: fixture.new_part_ids[6],
        },
    )
    .await
    .expect("update should not fail at sql layer")
    .expect("update should succeed while robot is queued");

    assert!(updated.pending);

    let states = list_robot_config_states(&pool, fixture.user_id)
        .await
        .expect("robot config states should load");
    let robot = states
        .iter()
        .find(|state| state.robot_id == fixture.robot_id)
        .expect("fixture robot should appear in config states");

    assert!(robot.change_pending);
    assert_eq!(robot.robot_name, "rust_bot");
    assert_eq!(robot.ore_container_id, fixture.new_part_ids[0]);
    assert_eq!(robot.memory_module_id, fixture.new_part_ids[3]);
    assert!(robot.ore_container_name.contains("-new-0"));
    assert!(robot.memory_module_name.contains("-new-3"));
    assert_eq!(robot.memory_size, 56);
    assert_eq!(robot.recharge_time, 14);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn list_robot_config_part_asset_states_counts_pending_parts_as_assigned() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db robots test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RobotConfigFixture::create(&pool, true, true, 8).await;

    update_robot_config(
        &pool,
        UpdateRobotConfigRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            robot_name: "rust_bot".to_string(),
            program_source_id: fixture.program_source_id,
            ore_container_id: fixture.new_part_ids[0],
            mining_unit_id: fixture.new_part_ids[1],
            battery_id: fixture.new_part_ids[2],
            memory_module_id: fixture.new_part_ids[3],
            cpu_id: fixture.new_part_ids[4],
            engine_id: fixture.new_part_ids[5],
            ore_scanner_id: fixture.new_part_ids[6],
        },
    )
    .await
    .expect("update should not fail at sql layer")
    .expect("update should succeed");

    let assets = list_robot_config_part_asset_states(&pool, fixture.user_id)
        .await
        .expect("robot part asset states should load");

    let new_memory = assets
        .iter()
        .find(|asset| asset.robot_part_id == fixture.new_part_ids[3])
        .expect("new memory module should appear in asset states");
    assert_eq!(new_memory.unassigned, 0);

    let old_memory = assets
        .iter()
        .find(|asset| asset.robot_part_id == fixture.current_part_ids[3])
        .expect("old memory module should appear in asset states");
    assert_eq!(
        old_memory.unassigned, 0,
        "active Robot row still references old parts until pending changes commit"
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn robot_stats_loaders_return_header_ore_and_area_totals() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db robots stats test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RobotConfigFixture::create(&pool, true, true, 8).await;
    let mining_area_id = fixture
        .mining_area_id
        .expect("queued fixture should create a mining area");

    sqlx::query("UPDATE Robot SET totalMiningRuns = 5 WHERE id = ?")
        .bind(fixture.robot_id)
        .execute(&pool)
        .await
        .expect("failed to set total mining runs");
    sqlx::query(
        "INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax) VALUES (?, ?, 40, 8)",
    )
    .bind(fixture.robot_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert lifetime ore");
    sqlx::query(
        "INSERT INTO RobotMiningAreaScore (robotId, miningAreaId, totalRuns, score) \
         VALUES (?, ?, 3, 21.5)",
    )
    .bind(fixture.robot_id)
    .bind(mining_area_id)
    .execute(&pool)
    .await
    .expect("failed to insert area score");

    let header = load_robot_stats_header(&pool, fixture.robot_id)
        .await
        .expect("header load should not fail")
        .expect("robot header should exist");
    assert_eq!(header.robot_id, fixture.robot_id);
    assert_eq!(header.total_mining_runs, 5);
    assert!(!header.username.is_empty());

    let ore_stats = list_robot_lifetime_ore_stats(&pool, fixture.robot_id)
        .await
        .expect("ore stats should load");
    assert_eq!(ore_stats.len(), 1);
    assert_eq!(ore_stats[0].ore_id, fixture.ore_id);
    assert_eq!(ore_stats[0].amount, 40);
    assert_eq!(ore_stats[0].tax, 8);

    let area_stats = list_robot_mining_area_stats(&pool, fixture.robot_id)
        .await
        .expect("area stats should load");
    assert_eq!(area_stats.len(), 1);
    assert_eq!(area_stats[0].mining_area_id, mining_area_id);
    assert_eq!(area_stats[0].total_runs, 3);
    assert!((area_stats[0].score - 21.5).abs() < f64::EPSILON);

    assert!(
        load_robot_stats_header(&pool, i64::MAX)
            .await
            .expect("missing header load should not fail")
            .is_none()
    );

    fixture.cleanup(&pool).await;
}
