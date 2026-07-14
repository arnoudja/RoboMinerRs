use robominer_db::MySqlPool;

use crate::{insert_row_id, unique_prefix};

pub async fn ensure_default_robot_parts(pool: &MySqlPool) {
    let existing_default_parts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM RobotPart WHERE id IN (101, 201, 301, 401, 501, 601, 701)",
    )
    .fetch_one(pool)
    .await
    .expect("failed to count default robot parts");
    if existing_default_parts >= 7 {
        return;
    }

    let ore_id = insert_row_id(
        pool,
        sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!(
            "{}-ore",
            unique_prefix("rust-default")
        )),
    )
    .await;
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

    for (robot_part_id, type_id, scan_time, scan_distance) in [
        (101, 1, 0, 0),
        (201, 2, 0, 0),
        (301, 3, 0, 0),
        (401, 4, 0, 0),
        (501, 5, 0, 0),
        (601, 6, 0, 0),
        (701, 7, 6, 5),
    ] {
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
