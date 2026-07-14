use robominer_db::MySqlPool;

use crate::{insert_row_id, insert_user_with_credentials, unique_prefix};

pub struct ShopCatalog {
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub robot_part_type_id: i64,
    pub robot_part_id: i64,
}

pub struct ShopFixture {
    pub user_id: i64,
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub robot_part_type_id: i64,
    pub robot_part_id: i64,
    pub robot_id: Option<i64>,
}

impl ShopFixture {
    pub async fn create(
        pool: &MySqlPool,
        user_ore_amount: i32,
        robot_part_cost: i32,
        robot_part_total_owned: i32,
        use_robot_part: bool,
    ) -> Self {
        let prefix = unique_prefix("rust-shop");
        let catalog = insert_shop_catalog(pool, &prefix, robot_part_cost).await;
        let user_id = insert_user_with_credentials(
            pool,
            &format!("{prefix}-user"),
            &format!("{prefix}@example.invalid"),
            "test-password",
        )
        .await;
        attach_user_shop_assets(
            pool,
            user_id,
            &catalog,
            user_ore_amount,
            robot_part_total_owned,
        )
        .await;
        let robot_id = if use_robot_part {
            Some(insert_shop_robot(pool, user_id, &prefix, catalog.robot_part_id).await)
        } else {
            None
        };

        Self {
            user_id,
            ore_id: catalog.ore_id,
            ore_price_id: catalog.ore_price_id,
            robot_part_type_id: catalog.robot_part_type_id,
            robot_part_id: catalog.robot_part_id,
            robot_id,
        }
    }

    pub async fn attach_to_user(
        pool: &MySqlPool,
        prefix: &str,
        user_id: i64,
        user_ore_amount: i32,
        robot_part_cost: i32,
        robot_part_total_owned: i32,
    ) -> Self {
        let catalog = insert_shop_catalog(pool, prefix, robot_part_cost).await;
        attach_user_shop_assets(
            pool,
            user_id,
            &catalog,
            user_ore_amount,
            robot_part_total_owned,
        )
        .await;

        Self {
            user_id,
            ore_id: catalog.ore_id,
            ore_price_id: catalog.ore_price_id,
            robot_part_type_id: catalog.robot_part_type_id,
            robot_part_id: catalog.robot_part_id,
            robot_id: None,
        }
    }

    pub async fn assert_ore_amount(&self, pool: &MySqlPool, expected_amount: i32) {
        let amount: i32 =
            sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
                .bind(self.user_id)
                .bind(self.ore_id)
                .fetch_one(pool)
                .await
                .expect("failed to load user ore asset");
        assert_eq!(amount, expected_amount);
    }

    pub async fn assert_robot_part_total_owned(
        &self,
        pool: &MySqlPool,
        expected_total_owned: Option<i32>,
    ) {
        let total_owned: Option<i32> = sqlx::query_scalar(
            "SELECT totalOwned FROM UserRobotPartAsset WHERE userId = ? AND robotPartId = ?",
        )
        .bind(self.user_id)
        .bind(self.robot_part_id)
        .fetch_optional(pool)
        .await
        .expect("failed to load user robot part asset");
        assert_eq!(total_owned, expected_total_owned);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        if let Some(robot_id) = self.robot_id {
            let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
                .bind(robot_id)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM User WHERE id = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        cleanup_shop_catalog(pool, &self.catalog_ids()).await;
    }

    pub async fn cleanup_attached(&self, pool: &MySqlPool, delete_user: bool) {
        let _ = sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        if delete_user {
            let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
                .bind(self.user_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM User WHERE id = ?")
                .bind(self.user_id)
                .execute(pool)
                .await;
        }
        cleanup_shop_catalog(pool, &self.catalog_ids()).await;
    }

    fn catalog_ids(&self) -> ShopCatalog {
        ShopCatalog {
            ore_id: self.ore_id,
            ore_price_id: self.ore_price_id,
            robot_part_type_id: self.robot_part_type_id,
            robot_part_id: self.robot_part_id,
        }
    }
}

pub async fn insert_shop_catalog(
    pool: &MySqlPool,
    prefix: &str,
    robot_part_cost: i32,
) -> ShopCatalog {
    let ore_id = insert_row_id(
        pool,
        sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
    )
    .await;
    let ore_price_id = insert_row_id(
        pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
            .bind(format!("{prefix}-price")),
    )
    .await;
    insert_row_id(
        pool,
        sqlx::query("INSERT INTO OrePriceAmount (orePriceId, oreId, amount) VALUES (?, ?, ?)")
            .bind(ore_price_id)
            .bind(ore_id)
            .bind(robot_part_cost),
    )
    .await;

    let robot_part_type_id = insert_row_id(
        pool,
        sqlx::query("INSERT INTO RobotPartType (typeName) VALUES (?)")
            .bind(format!("{prefix}-type")),
    )
    .await;
    let robot_part_id = insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO RobotPart \
             (typeId, tierId, partName, orePriceId, weight, volume, powerUsage) \
             VALUES (?, ?, ?, ?, 1, 1, 1)",
        )
        .bind(robot_part_type_id)
        .bind(ore_id)
        .bind(format!("{prefix}-part"))
        .bind(ore_price_id),
    )
    .await;

    ShopCatalog {
        ore_id,
        ore_price_id,
        robot_part_type_id,
        robot_part_id,
    }
}

pub async fn attach_user_shop_assets(
    pool: &MySqlPool,
    user_id: i64,
    catalog: &ShopCatalog,
    user_ore_amount: i32,
    robot_part_total_owned: i32,
) {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
             VALUES (?, ?, ?, 100)",
        )
        .bind(user_id)
        .bind(catalog.ore_id)
        .bind(user_ore_amount),
    )
    .await;

    if robot_part_total_owned > 0 {
        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO UserRobotPartAsset (userId, robotPartId, totalOwned) \
                 VALUES (?, ?, ?)",
            )
            .bind(user_id)
            .bind(catalog.robot_part_id)
            .bind(robot_part_total_owned),
        )
        .await;
    }
}

async fn insert_shop_robot(
    pool: &MySqlPool,
    user_id: i64,
    prefix: &str,
    robot_part_id: i64,
) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO Robot \
             (userId, robotName, sourceCode, oreContainerId, rechargeTime, maxOre, \
              miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, \
              backwardSpeed, rotateSpeed, robotSize, rechargeEndTime, miningEndTime) \
             VALUES (?, ?, 'mine();', ?, 1, 100, 4, 1, 128, 1, 1.0, 1.0, 90, 1.0, \
                     TIMESTAMPADD(SECOND, -10, NOW()), NULL)",
        )
        .bind(user_id)
        .bind(format!("{prefix}-robot"))
        .bind(robot_part_id),
    )
    .await
}

async fn cleanup_shop_catalog(pool: &MySqlPool, catalog: &ShopCatalog) {
    let _ = sqlx::query("DELETE FROM RobotPart WHERE id = ?")
        .bind(catalog.robot_part_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM RobotPartType WHERE id = ?")
        .bind(catalog.robot_part_type_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePriceAmount WHERE orePriceId = ?")
        .bind(catalog.ore_price_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(catalog.ore_price_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
        .bind(catalog.ore_id)
        .execute(pool)
        .await;
}
