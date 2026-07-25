use std::path::PathBuf;

use anyhow::{Result, ensure};

use super::ensure_positive_user_id;
use crate::cli::ShopCommand;
use crate::database::connect_database;
use crate::shop::{buy_robot_part, sell_robot_part, shop_catalog_states, shop_robot_part_states};

pub(crate) async fn dispatch_shop(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: ShopCommand,
) -> Result<()> {
    match command {
        ShopCommand::Buy {
            user_id,
            robot_part_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                robot_part_id > 0,
                "--robot-part-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            buy_robot_part(
                &pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await
        }
        ShopCommand::Sell {
            user_id,
            robot_part_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                robot_part_id > 0,
                "--robot-part-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            sell_robot_part(
                &pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await
        }
        ShopCommand::RobotPartStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            shop_robot_part_states(&pool, user_id).await
        }
        ShopCommand::CatalogStates => {
            let pool = connect_database(database_url, config).await?;
            shop_catalog_states(&pool).await
        }
    }
}
