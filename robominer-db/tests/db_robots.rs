use robominer_db::{
    UpdateRobotConfigRequest, list_robot_config_part_asset_states, list_robot_config_states,
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
