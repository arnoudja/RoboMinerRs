use robominer_db::{
    get_mining_area, list_mining_area_ore_supplies, list_mining_area_overview_areas_for_user,
    list_mining_area_overview_ores_for_user, list_mining_area_overview_percentages_for_user,
};
use robominer_test_support::{
    insert_area_supply, insert_mining_area, insert_ore, insert_ore_price, insert_robot,
    insert_row_id, insert_user, unique_prefix,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn list_mining_area_ore_supplies_returns_seeded_rows() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db mining areas test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-area-supply");
    let primary_ore_id = insert_ore(&pool, &format!("{prefix}-primary")).await;
    let secondary_ore_id = insert_ore(&pool, &format!("{prefix}-secondary")).await;
    let ore_price_id = insert_ore_price(&pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(&pool, &prefix).await;
    let ai_robot_id = insert_robot(&pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let mining_area_id =
        insert_mining_area(&pool, &prefix, ore_price_id, ai_robot_id, 0).await;
    insert_area_supply(&pool, mining_area_id, primary_ore_id, 12, 2).await;
    insert_area_supply(&pool, mining_area_id, secondary_ore_id, 3, 1).await;

    let supplies = list_mining_area_ore_supplies(&pool, mining_area_id)
        .await
        .expect("ore supplies should load");

    assert_eq!(supplies.len(), 2);
    assert_eq!(supplies[0].ore_id, primary_ore_id);
    assert_eq!(supplies[0].supply, 12);
    assert_eq!(supplies[0].radius, 2);
    assert_eq!(supplies[1].ore_id, secondary_ore_id);
    assert_eq!(supplies[1].supply, 3);

    let area = get_mining_area(&pool, mining_area_id)
        .await
        .expect("mining area should load")
        .expect("mining area should exist");
    assert!(area.area_name.contains(&format!("{prefix}-area")));

    cleanup_area_fixture(
        &pool,
        mining_area_id,
        ai_robot_id,
        user_id,
        ore_price_id,
        &[primary_ore_id, secondary_ore_id],
    )
    .await;
}

#[tokio::test]
#[serial]
async fn mining_area_overview_helpers_respect_user_area_grant() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db mining areas test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-area-grant");
    let granted_ore_id = insert_ore(&pool, &format!("{prefix}-granted")).await;
    let hidden_ore_id = insert_ore(&pool, &format!("{prefix}-hidden")).await;
    let ore_price_id = insert_ore_price(&pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(&pool, &prefix).await;
    let ai_robot_id = insert_robot(&pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let granted_area_id =
        insert_mining_area(&pool, &format!("{prefix}-granted"), ore_price_id, ai_robot_id, 0)
            .await;
    let hidden_area_id =
        insert_mining_area(&pool, &format!("{prefix}-hidden"), ore_price_id, ai_robot_id, 0)
            .await;

    insert_area_supply(&pool, granted_area_id, granted_ore_id, 8, 1).await;
    insert_area_supply(&pool, hidden_area_id, hidden_ore_id, 5, 1).await;

    sqlx::query("INSERT INTO UserMiningArea (userId, miningAreaId) VALUES (?, ?)")
        .bind(user_id)
        .bind(granted_area_id)
        .execute(&pool)
        .await
        .expect("failed to grant mining area");

    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningAreaLifetimeResult \
             (miningAreaId, oreId, totalAmount, totalContainerSize) \
             VALUES (?, ?, 40, 100)",
        )
        .bind(granted_area_id)
        .bind(granted_ore_id),
    )
    .await;
    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningAreaLifetimeResult \
             (miningAreaId, oreId, totalAmount, totalContainerSize) \
             VALUES (?, ?, 10, 100)",
        )
        .bind(hidden_area_id)
        .bind(hidden_ore_id),
    )
    .await;

    let overview_areas = list_mining_area_overview_areas_for_user(&pool, user_id)
        .await
        .expect("overview areas should load");
    assert_eq!(overview_areas.len(), 1);
    assert_eq!(overview_areas[0].mining_area_id, granted_area_id);
    assert!((overview_areas[0].total_percentage - 40.0).abs() < f64::EPSILON);

    let overview_ores = list_mining_area_overview_ores_for_user(&pool, user_id)
        .await
        .expect("overview ores should load");
    assert!(
        overview_ores
            .iter()
            .any(|ore| ore.ore_id == granted_ore_id),
        "expected granted-area ore in user overview"
    );
    assert!(
        !overview_ores.iter().any(|ore| ore.ore_id == hidden_ore_id),
        "hidden-area ore should stay out of user overview"
    );

    let overview_percentages = list_mining_area_overview_percentages_for_user(&pool, user_id)
        .await
        .expect("overview percentages should load");
    assert_eq!(overview_percentages.len(), 1);
    assert_eq!(overview_percentages[0].mining_area_id, granted_area_id);
    assert_eq!(overview_percentages[0].ore_id, granted_ore_id);
    assert!((overview_percentages[0].percentage - 40.0).abs() < f64::EPSILON);

    cleanup_granted_area_fixture(
        &pool,
        granted_area_id,
        hidden_area_id,
        ai_robot_id,
        user_id,
        ore_price_id,
        granted_ore_id,
        hidden_ore_id,
    )
    .await;
}

async fn cleanup_area_fixture(
    pool: &robominer_db::MySqlPool,
    mining_area_id: i64,
    ai_robot_id: i64,
    user_id: i64,
    ore_price_id: i64,
    ore_ids: &[i64],
) {
    let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId = ?")
        .bind(mining_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
        .bind(ai_robot_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(ore_price_id)
        .execute(pool)
        .await;
    for ore_id in ore_ids {
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(ore_id)
            .execute(pool)
            .await;
    }
}

async fn cleanup_granted_area_fixture(
    pool: &robominer_db::MySqlPool,
    granted_area_id: i64,
    hidden_area_id: i64,
    ai_robot_id: i64,
    user_id: i64,
    ore_price_id: i64,
    granted_ore_id: i64,
    hidden_ore_id: i64,
) {
    let _ = sqlx::query("DELETE FROM MiningAreaLifetimeResult WHERE miningAreaId IN (?, ?)")
        .bind(granted_area_id)
        .bind(hidden_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserMiningArea WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId IN (?, ?)")
        .bind(granted_area_id)
        .bind(hidden_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id IN (?, ?)")
        .bind(granted_area_id)
        .bind(hidden_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
        .bind(ai_robot_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(ore_price_id)
        .execute(pool)
        .await;
    for ore_id in [granted_ore_id, hidden_ore_id] {
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(ore_id)
            .execute(pool)
            .await;
    }
}
