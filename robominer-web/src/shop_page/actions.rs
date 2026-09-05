//! Shop buy/sell mutations for the shop page.

pub(super) async fn apply_shop_mutations(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    buy_part_id: Option<i64>,
    sell_part_id: Option<i64>,
    sell_all_unassigned: bool,
) -> Result<Option<String>, crate::page_context::PageLoadError> {
    if let Some(robot_part_id) = buy_part_id {
        return Ok(Some(
            match robominer_db::shop::buy_robot_part(
                pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await?
            {
                robominer_db::DbOutcome::Success(_) => "Robot part bought".to_string(),
                robominer_db::DbOutcome::Rejected(rejection) => format!(
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
            match robominer_db::sell_all_unassigned_robot_parts(pool, user_id).await? {
                robominer_db::DbOutcome::Success(result) => {
                    if result.sold_count == 1 {
                        "Sold 1 unassigned robot part".to_string()
                    } else {
                        format!("Sold {} unassigned robot parts", result.sold_count)
                    }
                }
                robominer_db::DbOutcome::Rejected(rejection) => format!(
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
            match robominer_db::shop::sell_robot_part(
                pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await?
            {
                robominer_db::DbOutcome::Success(_) => "Robot part sold".to_string(),
                robominer_db::DbOutcome::Rejected(rejection) => format!(
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
