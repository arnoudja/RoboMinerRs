use crate::output::escape_state_field;
use anyhow::{Context, Result, anyhow};

pub(crate) async fn buy_robot_part(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::RobotPartTransactionRequest,
) -> Result<()> {
    match robominer_domain::buy_robot_part(pool, request)
        .await
        .context("failed to buy robot part")?
    {
        Ok(result) => {
            println!(
                "Bought robot part {} for user {}",
                result.robot_part_id, request.user_id
            );
            Ok(())
        }
        Err(rejection) => Err(anyhow!(
            "unable to buy robot part: {}",
            robominer_domain::robot_part_transaction_rejection_message(rejection)
        )),
    }
}

pub(crate) async fn sell_robot_part(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::RobotPartTransactionRequest,
) -> Result<()> {
    match robominer_domain::sell_robot_part(pool, request)
        .await
        .context("failed to sell robot part")?
    {
        Ok(result) => {
            println!(
                "Sold robot part {} for user {}",
                result.robot_part_id, request.user_id
            );
            Ok(())
        }
        Err(rejection) => Err(anyhow!(
            "unable to sell robot part: {}",
            robominer_domain::robot_part_transaction_rejection_message(rejection)
        )),
    }
}

pub(crate) async fn shop_robot_part_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<()> {
    let states = robominer_domain::list_shop_robot_part_states(pool, user_id)
        .await
        .context("failed to load shop robot part states")?;

    for state in states {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            state.robot_part_id,
            state.total_owned,
            state.assigned,
            state.unassigned,
            state.can_buy,
            state.can_sell
        );
    }

    Ok(())
}

pub(crate) async fn shop_catalog_states(pool: &robominer_db::MySqlPool) -> Result<()> {
    let ores = robominer_domain::list_shop_catalog_ores(pool)
        .await
        .context("failed to load shop ores")?;
    let robot_part_types = robominer_domain::list_shop_catalog_robot_part_types(pool)
        .await
        .context("failed to load shop robot part types")?;
    let robot_parts = robominer_domain::list_shop_catalog_robot_parts(pool)
        .await
        .context("failed to load shop robot parts")?;
    let costs = robominer_domain::list_shop_catalog_robot_part_costs(pool)
        .await
        .context("failed to load shop robot part costs")?;

    for ore in ores {
        println!("O\t{}\t{}", ore.id, escape_state_field(&ore.ore_name));
    }

    for robot_part_type in robot_part_types {
        println!(
            "T\t{}\t{}",
            robot_part_type.id,
            escape_state_field(&robot_part_type.type_name)
        );
    }

    for robot_part in robot_parts {
        println!(
            "P\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            robot_part.robot_part_id,
            robot_part.type_id,
            robot_part.tier_id,
            escape_state_field(&robot_part.tier_name),
            escape_state_field(&robot_part.part_name),
            robot_part.ore_capacity,
            robot_part.mining_capacity,
            robot_part.battery_capacity,
            robot_part.memory_capacity,
            robot_part.cpu_capacity,
            robot_part.forward_capacity,
            robot_part.backward_capacity,
            robot_part.rotate_capacity,
            robot_part.recharge_time,
            robot_part.weight,
            robot_part.volume,
            robot_part.power_usage
        );
    }

    for cost in costs {
        println!(
            "C\t{}\t{}\t{}\t{}",
            cost.robot_part_id,
            cost.ore_id,
            escape_state_field(&cost.ore_name),
            cost.amount
        );
    }

    Ok(())
}
