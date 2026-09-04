use robominer_db::{DEFAULT_PART_IDS, MySqlPool};

use crate::{insert_ore, insert_row_id, unique_prefix};

fn default_part_id_list_sql() -> String {
    DEFAULT_PART_IDS
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub async fn ensure_default_robot_parts(pool: &MySqlPool) {
    let existing_default_parts: i64 = sqlx::query_scalar(robominer_db::assert_sql_safe(format!(
        "SELECT COUNT(*) FROM RobotPart WHERE id IN ({})",
        default_part_id_list_sql()
    )))
    .fetch_one(pool)
    .await
    .expect("failed to count default robot parts");
    if existing_default_parts >= DEFAULT_PART_IDS.len() as i64 {
        return;
    }

    let ore_id = insert_ore(pool, &format!("{}-ore", unique_prefix("rust-default"))).await;
    let ore_price_id = insert_row_id(
        pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES ('rust-default-price')"),
    )
    .await;

    for type_id in 1..=7 {
        sqlx::query("INSERT IGNORE INTO RobotPartType (id, typeName) VALUES (?, ?)")
            .bind(type_id)
            .bind(format!("default-type-{type_id}"))
            .execute(pool)
            .await
            .expect("failed to ensure default part type");
    }

    let part_specs = [
        (DEFAULT_PART_IDS[0], 1, 0, 0),
        (DEFAULT_PART_IDS[1], 2, 0, 0),
        (DEFAULT_PART_IDS[2], 3, 0, 0),
        (DEFAULT_PART_IDS[3], 4, 0, 0),
        (DEFAULT_PART_IDS[4], 5, 0, 0),
        (DEFAULT_PART_IDS[5], 6, 0, 0),
        (DEFAULT_PART_IDS[6], 7, 6, 5),
    ];
    for (robot_part_id, type_id, scan_time, scan_distance) in part_specs {
        sqlx::query(
            "INSERT IGNORE INTO RobotPart \
             (id, typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
              batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, backwardCapacity, \
              rotateCapacity, rechargeTime, scanTime, scanDistance, weight, volume, powerUsage) \
             VALUES (?, ?, ?, ?, ?, 10, 1, 100, 8, 2, 50, 50, 50, 1, ?, ?, 10, 10, 1)",
        )
        .bind(robot_part_id)
        .bind(type_id)
        .bind(ore_id)
        .bind(format!("default-part-{robot_part_id}"))
        .bind(ore_price_id)
        .bind(scan_time)
        .bind(scan_distance)
        .execute(pool)
        .await
        .expect("failed to ensure default robot part");
    }
}
