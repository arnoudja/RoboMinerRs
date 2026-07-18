mod achievement;
mod activity;
mod assets;
mod cli;
mod database;
mod leaderboard;
mod migrate;
mod mining;
mod output;
mod program;
mod rally;
mod robot;
mod shop;
mod user;
mod verify;

use anyhow::{Result, ensure};
use clap::Parser;

use achievement::{achievement_page_states, achievement_states, claim_achievement_step};
use activity::{activity_states, rally_view_state};
use assets::user_ore_asset_states;
use cli::{Cli, Command};
use database::connect_database;
use leaderboard::leaderboard_states;
use migrate::{migrate, migrate_status};
use mining::{
    cancel_mining_queue, claim_results, enqueue_mining, mining_area_overview_states,
    mining_area_scores, mining_queue_page_states, mining_queue_states, mining_result_states,
};
use program::{
    create_program_source, delete_program_source, program_source_states, update_program_source,
};
use rally::{
    RunPoolOptions, RunRalliesOptions, RunRallyOptions, run_pool, run_rallies, run_rally,
    validate_run_pool_options, validate_run_rallies_options, validate_run_rally_options,
};
use robot::{robot_config_states, update_robot_config};
use shop::{buy_robot_part, sell_robot_part, shop_catalog_states, shop_robot_part_states};
use user::{account_state, create_user, update_user_account, verify_login, verify_user_password};
use verify::{
    SimulateSourceOptions, simulate_source_file, verify as verify_program, verify_source_file,
};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Verify { program_source_id } => {
            let pool = connect_database(cli.database_url, cli.config).await?;
            verify_program(&pool, program_source_id).await
        }
        Command::VerifySource { source_file } => verify_source_file(&source_file),
        Command::SimulateSource {
            source_file,
            robot,
            turns,
            size_x,
            size_y,
            ore_x,
            ore_y,
            ore_type,
            ore_amount,
            mining_speed,
            forward_speed,
            backward_speed,
            rotate_speed,
        } => simulate_source_file(SimulateSourceOptions {
            source_file,
            robot_files: robot,
            turns,
            size_x,
            size_y,
            ore_x,
            ore_y,
            ore_type,
            ore_amount,
            mining_speed,
            forward_speed,
            backward_speed,
            rotate_speed,
        }),
        Command::ClaimResults { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            claim_results(&pool, user_id).await
        }
        Command::EnqueueMining {
            user_id,
            robot_id,
            mining_area_id,
            fill,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(robot_id > 0, "--robot-id must be greater than zero");
            ensure!(
                mining_area_id > 0,
                "--mining-area-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            enqueue_mining(
                &pool,
                robominer_db::EnqueueMiningRequest {
                    user_id,
                    robot_id,
                    mining_area_id,
                    fill,
                },
            )
            .await
        }
        Command::CancelMiningQueue {
            user_id,
            mining_queue_id,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(
                mining_queue_id > 0,
                "--mining-queue-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            cancel_mining_queue(
                &pool,
                robominer_db::CancelMiningQueueRequest {
                    user_id,
                    mining_queue_id,
                },
            )
            .await
        }
        Command::MiningQueueStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            mining_queue_states(&pool, user_id).await
        }
        Command::MiningQueuePageStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            mining_queue_page_states(&pool, user_id).await
        }
        Command::ActivityStates {
            user_id,
            max_users,
            max_rallies,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(max_users > 0, "--max-users must be greater than zero");
            ensure!(max_rallies > 0, "--max-rallies must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            activity_states(&pool, max_users, max_rallies).await
        }
        Command::RallyViewState {
            user_id,
            rally_result_id,
            require_user_result,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(
                rally_result_id > 0,
                "--rally-result-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            rally_view_state(&pool, user_id, rally_result_id, require_user_result).await
        }
        Command::UserOreAssetStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            user_ore_asset_states(&pool, user_id).await
        }
        Command::MiningAreaScores { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            mining_area_scores(&pool, user_id).await
        }
        Command::MiningResultStates {
            user_id,
            max_results,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(max_results > 0, "--max-results must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            mining_result_states(&pool, user_id, max_results).await
        }
        Command::MiningAreaOverviewStates => {
            let pool = connect_database(cli.database_url, cli.config).await?;
            mining_area_overview_states(&pool).await
        }
        Command::BuyRobotPart {
            user_id,
            robot_part_id,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(
                robot_part_id > 0,
                "--robot-part-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            buy_robot_part(
                &pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await
        }
        Command::SellRobotPart {
            user_id,
            robot_part_id,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(
                robot_part_id > 0,
                "--robot-part-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            sell_robot_part(
                &pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await
        }
        Command::ShopRobotPartStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            shop_robot_part_states(&pool, user_id).await
        }
        Command::ShopCatalogStates => {
            let pool = connect_database(cli.database_url, cli.config).await?;
            shop_catalog_states(&pool).await
        }
        Command::RobotConfigStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            robot_config_states(&pool, user_id).await
        }
        Command::UpdateRobotConfig {
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
            ensure!(user_id > 0, "--user-id must be greater than zero");
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
            let pool = connect_database(cli.database_url, cli.config).await?;
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
        Command::CreateProgramSource {
            user_id,
            source_name,
            source_code,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            create_program_source(
                &pool,
                robominer_db::CreateProgramSourceRequest {
                    user_id,
                    source_name,
                    source_code,
                },
            )
            .await
        }
        Command::UpdateProgramSource {
            user_id,
            program_source_id,
            source_name,
            source_code,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(
                program_source_id > 0,
                "--program-source-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            update_program_source(
                &pool,
                robominer_db::ProgramSourceWriteRequest {
                    user_id,
                    program_source_id,
                    source_name,
                    source_code,
                },
            )
            .await
        }
        Command::DeleteProgramSource {
            user_id,
            program_source_id,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(
                program_source_id > 0,
                "--program-source-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            delete_program_source(&pool, user_id, program_source_id).await
        }
        Command::ProgramSourceStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            program_source_states(&pool, user_id).await
        }
        Command::AccountState { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            account_state(&pool, user_id).await
        }
        Command::CreateUser {
            username,
            email,
            password,
        } => {
            ensure!(!username.is_empty(), "--username must not be empty");
            ensure!(!email.is_empty(), "--email must not be empty");
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(cli.database_url, cli.config).await?;
            create_user(
                &pool,
                robominer_db::CreateUserRequest {
                    username,
                    email,
                    password,
                },
            )
            .await
        }
        Command::UpdateUserAccount {
            user_id,
            username,
            email,
            password,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(!username.is_empty(), "--username must not be empty");
            ensure!(!email.is_empty(), "--email must not be empty");
            if let Some(password) = &password {
                ensure!(!password.is_empty(), "--password must not be empty");
            }
            let pool = connect_database(cli.database_url, cli.config).await?;
            update_user_account(
                &pool,
                robominer_db::UpdateUserAccountRequest {
                    user_id,
                    username,
                    email,
                    password,
                },
            )
            .await
        }
        Command::VerifyLogin {
            login_name,
            password,
        } => {
            ensure!(!login_name.is_empty(), "--login-name must not be empty");
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(cli.database_url, cli.config).await?;
            verify_login(
                &pool,
                robominer_db::VerifyLoginRequest {
                    login_name,
                    password,
                },
            )
            .await
        }
        Command::VerifyUserPassword { user_id, password } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(cli.database_url, cli.config).await?;
            verify_user_password(
                &pool,
                robominer_db::VerifyUserPasswordRequest { user_id, password },
            )
            .await
        }
        Command::ClaimAchievementStep {
            user_id,
            achievement_id,
        } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            ensure!(
                achievement_id > 0,
                "--achievement-id must be greater than zero"
            );
            let pool = connect_database(cli.database_url, cli.config).await?;
            claim_achievement_step(
                &pool,
                robominer_db::ClaimAchievementStepRequest {
                    user_id,
                    achievement_id,
                },
            )
            .await
        }
        Command::AchievementStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            achievement_states(&pool, user_id).await
        }
        Command::AchievementPageStates { user_id } => {
            ensure!(user_id > 0, "--user-id must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            achievement_page_states(&pool, user_id).await
        }
        Command::LeaderboardStates { max_entries } => {
            ensure!(max_entries > 0, "--max-entries must be greater than zero");
            let pool = connect_database(cli.database_url, cli.config).await?;
            leaderboard_states(&pool, max_entries).await
        }
        Command::RunRally {
            mining_area_id,
            seed,
            persist,
            result_data_file,
        } => {
            let options = RunRallyOptions {
                mining_area_id,
                seed,
                persist,
                result_data_file,
            };
            validate_run_rally_options(&options)?;

            let pool = connect_database(cli.database_url, cli.config).await?;
            run_rally(&pool, options).await.map(|_| ())
        }
        Command::RunPool {
            pool_id,
            seed,
            persist,
            until_complete,
            max_rallies,
        } => {
            let options = RunPoolOptions {
                pool_id,
                seed,
                persist,
                until_complete,
                max_rallies,
            };
            validate_run_pool_options(&options)?;

            let pool = connect_database(cli.database_url, cli.config).await?;
            run_pool(&pool, options).await.map(|_| ())
        }
        Command::RunRallies {
            once,
            loop_mode,
            sleep_seconds,
            seed,
            persist,
        } => {
            let options = RunRalliesOptions {
                once,
                loop_mode,
                sleep_seconds,
                seed,
                persist,
            };
            validate_run_rallies_options(&options)?;

            let pool = connect_database(cli.database_url, cli.config).await?;
            run_rallies(&pool, options).await
        }
        Command::Migrate => {
            let pool = connect_database(cli.database_url, cli.config).await?;
            migrate(&pool).await
        }
        Command::MigrateStatus { check } => {
            let pool = connect_database(cli.database_url, cli.config).await?;
            migrate_status(&pool, check).await
        }
    }
}
