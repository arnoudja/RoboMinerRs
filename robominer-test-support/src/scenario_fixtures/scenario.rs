use robominer_db::MySqlPool;

use crate::{
    ensure_base_catalog, insert_robot, insert_user_ore_asset, insert_user_with_credentials,
    unique_prefix,
};

/// Composable integration-test scenario: registered user, default robot, and ore wallet.
pub struct Scenario {
    pub prefix: String,
    pub user_id: i64,
    pub robot_id: i64,
    pub ore_id: i64,
}

impl Scenario {
    pub async fn user_with_robot_and_wallet(pool: &MySqlPool) -> Self {
        let prefix = unique_prefix("scenario");
        let user_id = insert_user_with_credentials(
            pool,
            &format!("{prefix}-user"),
            &format!("{prefix}@example.invalid"),
            "test-password-1",
        )
        .await;
        let catalog = ensure_base_catalog(pool, &prefix, 1).await;
        insert_user_ore_asset(pool, user_id, catalog.ore_id, 25, 1000).await;
        let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;

        Self {
            prefix,
            user_id,
            robot_id,
            ore_id: catalog.ore_id,
        }
    }
}
