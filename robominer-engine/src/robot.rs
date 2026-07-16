use crate::output::escape_state_field;
use anyhow::{Context, Result, anyhow};

pub(crate) async fn robot_config_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<()> {
    let program_sources = robominer_db::list_program_sources_for_user(pool, user_id)
        .await
        .context("failed to load robot configuration program sources")?;
    let robots = robominer_db::list_robot_config_states(pool, user_id)
        .await
        .context("failed to load robot configuration states")?;
    let part_assets = robominer_db::list_robot_config_part_asset_states(pool, user_id)
        .await
        .context("failed to load robot configuration part assets")?;

    for program_source in program_sources {
        println!(
            "P\t{}\t{}\t{}",
            program_source.id,
            escape_state_field(&program_source.source_name),
            program_source.compiled_size
        );
    }

    for robot in robots {
        println!(
            "R\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            robot.robot_id,
            escape_state_field(&robot.robot_name),
            robot.program_source_id,
            robot.ore_container_id,
            escape_state_field(&robot.ore_container_name),
            robot.mining_unit_id,
            escape_state_field(&robot.mining_unit_name),
            robot.battery_id,
            escape_state_field(&robot.battery_name),
            robot.memory_module_id,
            escape_state_field(&robot.memory_module_name),
            robot.cpu_id,
            escape_state_field(&robot.cpu_name),
            robot.engine_id,
            escape_state_field(&robot.engine_name),
            robot.ore_scanner_id,
            escape_state_field(&robot.ore_scanner_name),
            robot.recharge_time,
            robot.max_ore,
            robot.mining_speed,
            robot.max_turns,
            robot.memory_size,
            robot.cpu_speed,
            robot.forward_speed,
            robot.backward_speed,
            robot.rotate_speed,
            robot.robot_size,
            robot.scan_time,
            robot.scan_distance,
            robot.change_pending
        );
    }

    for part_asset in part_assets {
        println!(
            "A\t{}\t{}\t{}\t{}\t{}",
            part_asset.type_id,
            part_asset.robot_part_id,
            escape_state_field(&part_asset.part_name),
            part_asset.memory_capacity,
            part_asset.unassigned
        );
    }

    Ok(())
}

pub(crate) async fn update_robot_config(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::UpdateRobotConfigRequest,
) -> Result<()> {
    match robominer_db::update_robot_config(pool, request.clone())
        .await
        .context("failed to update robot configuration")?
    {
        Ok(result) => {
            let mode = if result.pending { "pending" } else { "active" };
            println!("Updated {mode} configuration for robot {}", result.robot_id);
            Ok(())
        }
        Err(rejection) => Err(anyhow!(
            "unable to update robot configuration: {}",
            robominer_domain::update_robot_config_rejection_cli_message(rejection)
        )),
    }
}
