//! Shop buy/sell mutations for the shop page.

use robominer_domain::{
    BuyRobotPartOutcome, DomainError, SellAllUnassignedRobotPartsOutcome, SellRobotPartOutcome,
};

fn shop_mutation_error(error: DomainError) -> crate::page_context::PageLoadError {
    crate::page_context::PageLoadError::from_database(error).unwrap_or_else(|_| {
        crate::page_context::PageLoadError::from(sqlx::Error::Configuration(
            "unexpected domain error on shop mutation".into(),
        ))
    })
}

pub(super) async fn apply_shop_mutations(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    buy_part_id: Option<i64>,
    sell_part_id: Option<i64>,
    sell_all_unassigned: bool,
) -> Result<Option<String>, crate::page_context::PageLoadError> {
    if let Some(robot_part_id) = buy_part_id {
        return Ok(Some(
            match robominer_domain::buy_robot_part(
                pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await
            .map_err(shop_mutation_error)?
            {
                BuyRobotPartOutcome::Success(_) => "Robot part bought".to_string(),
                BuyRobotPartOutcome::Rejected(rejection) => format!(
                    "Unable to buy robot part: {}",
                    robominer_domain::rejection_messages::robot_part_transaction_rejection_message(
                        rejection
                    )
                ),
            },
        ));
    }

    if sell_all_unassigned {
        return Ok(Some(
            match robominer_domain::sell_all_unassigned_robot_parts(pool, user_id)
                .await
                .map_err(shop_mutation_error)?
            {
                SellAllUnassignedRobotPartsOutcome::Success(result) => {
                    if result.sold_count == 1 {
                        "Sold 1 unassigned robot part".to_string()
                    } else {
                        format!("Sold {} unassigned robot parts", result.sold_count)
                    }
                }
                SellAllUnassignedRobotPartsOutcome::Rejected(rejection) => format!(
                    "Unable to sell robot parts: {}",
                    robominer_domain::rejection_messages::robot_part_transaction_rejection_message(
                        rejection
                    )
                ),
            },
        ));
    }

    if let Some(robot_part_id) = sell_part_id {
        return Ok(Some(
            match robominer_domain::sell_robot_part(
                pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await
            .map_err(shop_mutation_error)?
            {
                SellRobotPartOutcome::Success(_) => "Robot part sold".to_string(),
                SellRobotPartOutcome::Rejected(rejection) => format!(
                    "Unable to sell robot part: {}",
                    robominer_domain::rejection_messages::robot_part_transaction_rejection_message(
                        rejection
                    )
                ),
            },
        ));
    }

    Ok(None)
}
