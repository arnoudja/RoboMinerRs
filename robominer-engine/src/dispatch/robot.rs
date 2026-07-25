use std::path::PathBuf;

use anyhow::{Result, ensure};

use super::ensure_positive_user_id;
use crate::cli::RobotCommand;
use crate::database::connect_database;
use crate::robot::{robot_config_states, update_robot_config};

pub(crate) async fn dispatch_robot(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: RobotCommand,
) -> Result<()> {
    match command {
        RobotCommand::ConfigStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            robot_config_states(&pool, user_id).await
        }
        RobotCommand::UpdateConfig {
            user_id,
            robot_id,
            robot_name,
            program_source_id,
            ore_container_id,
            mining_unit_id,
            battery_id,
            memory_module_id,
            cpu_id,
            engine_id,
            ore_scanner_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(robot_id > 0, "--robot-id must be greater than zero");
            ensure!(
                program_source_id > 0,
                "--program-source-id must be greater than zero"
            );
            ensure!(
                ore_container_id > 0,
                "--ore-container-id must be greater than zero"
            );
            ensure!(
                mining_unit_id > 0,
                "--mining-unit-id must be greater than zero"
            );
            ensure!(battery_id > 0, "--battery-id must be greater than zero");
            ensure!(
                memory_module_id > 0,
                "--memory-module-id must be greater than zero"
            );
            ensure!(cpu_id > 0, "--cpu-id must be greater than zero");
            ensure!(engine_id > 0, "--engine-id must be greater than zero");
            ensure!(
                ore_scanner_id > 0,
                "--ore-scanner-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            update_robot_config(
                &pool,
                robominer_db::UpdateRobotConfigRequest {
                    user_id,
                    robot_id,
                    robot_name,
                    program_source_id,
                    ore_container_id,
                    mining_unit_id,
                    battery_id,
                    memory_module_id,
                    cpu_id,
                    engine_id,
                    ore_scanner_id,
                },
            )
            .await
        }
    }
}
