use robominer_domain::{
    load_next_rally_loadout, persist_rally_outcome, run_rally_loadout_with_animation_seed,
};
use robominer_test_support::RallyFixture;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn load_next_rally_loadout_applies_depot_capacity_and_banks_home_dump() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping depot loadout→rally DB test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RallyFixture::create(&pool).await;

    sqlx::query(
        "UPDATE Robot SET sourceCode = ?, robotSize = 1.5, miningSpeed = 9, cpuSpeed = 3 \
         WHERE id = ?",
    )
    .bind("mine(); dump(0);")
    .bind(fixture.queued_robot_id)
    .execute(&pool)
    .await
    .expect("failed to set dump-at-home program");

    sqlx::query("UPDATE MiningArea SET maxMoves = 20 WHERE id = ?")
        .bind(fixture.mining_area_id)
        .execute(&pool)
        .await
        .expect("failed to extend rally duration");

    sqlx::query(
        "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed, depotMaxAllowed) \
         VALUES (?, ?, 0, 100, 10)",
    )
    .bind(fixture.user_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("failed to unlock depot capacity");

    // Ensure the queue is rally-ready without waiting on miningTime.
    sqlx::query(
        "UPDATE MiningQueue SET creationTime = TIMESTAMPADD(SECOND, -3600, NOW()) WHERE id = ?",
    )
    .bind(fixture.mining_queue_id)
    .execute(&pool)
    .await
    .expect("failed to age mining queue row");

    let loadout = load_next_rally_loadout(&pool, fixture.mining_area_id)
        .await
        .expect("loadout query should succeed")
        .expect("queue should be rally-ready");

    assert_eq!(loadout.queue_entries.len(), 1);
    let mut expected_depot = [0; robominer_sim::MAX_ORE_TYPES];
    expected_depot[0] = 10;
    assert_eq!(
        loadout.queue_entries[0].robot.depot_capacity,
        expected_depot
    );
    assert_eq!(
        loadout.mining_area.ai_robot.depot_capacity,
        [0; robominer_sim::MAX_ORE_TYPES]
    );

    let run = run_rally_loadout_with_animation_seed(&loadout, 0).expect("rally should run");
    let player = &run.outcome.participants[0];
    assert!(
        player.ore[0] > 0,
        "depot-backed home dump should keep ore in the haul"
    );
    assert!(player.score > 0.0);

    let payload: serde_json::Value =
        serde_json::from_str(&run.result_data).expect("result data should be JSON");
    assert_eq!(payload["robots"]["robot"][0]["depotMaxA"], 10);
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("locations");
    let max_depot_a = locations
        .iter()
        .filter_map(|step| step["DA"].as_i64())
        .max()
        .unwrap_or(0);
    assert!(max_depot_a > 0, "animation should show depot fill");

    persist_rally_outcome(&pool, &loadout, &run.outcome, &run.result_data)
        .await
        .expect("persist should succeed");

    let ore_amount: i32 = sqlx::query_scalar(
        "SELECT amount FROM MiningOreResult WHERE miningQueueId = ? AND oreId = ?",
    )
    .bind(fixture.mining_queue_id)
    .bind(fixture.ore_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load persisted ore haul");
    assert_eq!(ore_amount, player.ore[0]);

    let rally_result_id: i64 =
        sqlx::query_scalar("SELECT rallyResultId FROM MiningQueue WHERE id = ?")
            .bind(fixture.mining_queue_id)
            .fetch_one(&pool)
            .await
            .expect("queue should reference rally result");
    let result_data: String = sqlx::query_scalar("SELECT resultData FROM RallyResult WHERE id = ?")
        .bind(rally_result_id)
        .fetch_one(&pool)
        .await
        .expect("failed to load persisted animation");
    assert!(result_data.contains(r#""depotMaxA":10"#));

    let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ? AND oreId = ?")
        .bind(fixture.user_id)
        .bind(fixture.ore_id)
        .execute(&pool)
        .await;
    fixture.cleanup(&pool).await;
}
