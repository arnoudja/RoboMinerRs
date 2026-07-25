use std::path::PathBuf;

use anyhow::{Result, ensure};

use crate::achievement::{achievement_page_states, achievement_states, claim_achievement_step};
use crate::activity::{activity_states, rally_view_state};
use crate::assets::user_ore_asset_states;
use crate::cli::{
    AchievementCommand, ActivityCommand, AssetsCommand, LeaderboardCommand, MigrateCommand,
    MiningCommand, ProgramCommand, RallyCommand, RobotCommand, ShopCommand, UserCommand,
};
use crate::database::connect_database;
use crate::leaderboard::leaderboard_states;
use crate::migrate::{migrate, migrate_status};
use crate::mining::{
    cancel_mining_queue, claim_results, enqueue_mining, mining_area_overview_states,
    mining_area_scores, mining_queue_page_states, mining_queue_states, mining_result_states,
};
use crate::program::{
    create_program_source, delete_program_source, program_source_states, update_program_source,
};
use crate::rally::{
    RunPoolOptions, RunRalliesOptions, RunRallyOptions, run_pool, run_rallies, run_rally,
    validate_run_pool_options, validate_run_rallies_options, validate_run_rally_options,
};
use crate::robot::{robot_config_states, update_robot_config};
use crate::shop::{buy_robot_part, sell_robot_part, shop_catalog_states, shop_robot_part_states};
use crate::user::{
    account_state, create_user, update_user_account, verify_login, verify_user_password,
};
use crate::verify::{
    SimulateSourceOptions, simulate_source_file, verify as verify_program, verify_source_file,
};

fn ensure_positive_user_id(user_id: i64) -> Result<()> {
    ensure!(user_id > 0, "--user-id must be greater than zero");
    Ok(())
}

pub(crate) async fn dispatch_mining(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: MiningCommand,
) -> Result<()> {
    match command {
        MiningCommand::ClaimResults { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            claim_results(&pool, user_id).await
        }
        MiningCommand::Enqueue {
            user_id,
            robot_id,
            mining_area_id,
            fill,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(robot_id > 0, "--robot-id must be greater than zero");
            ensure!(
                mining_area_id > 0,
                "--mining-area-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
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
        MiningCommand::CancelQueue {
            user_id,
            mining_queue_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                mining_queue_id > 0,
                "--mining-queue-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            cancel_mining_queue(
                &pool,
                robominer_db::CancelMiningQueueRequest {
                    user_id,
                    mining_queue_id,
                },
            )
            .await
        }
        MiningCommand::QueueStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            mining_queue_states(&pool, user_id).await
        }
        MiningCommand::QueuePageStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            mining_queue_page_states(&pool, user_id).await
        }
        MiningCommand::AreaScores { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            mining_area_scores(&pool, user_id).await
        }
        MiningCommand::ResultStates {
            user_id,
            max_results,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(max_results > 0, "--max-results must be greater than zero");
            let pool = connect_database(database_url, config).await?;
            mining_result_states(&pool, user_id, max_results).await
        }
        MiningCommand::AreaOverviewStates => {
            let pool = connect_database(database_url, config).await?;
            mining_area_overview_states(&pool).await
        }
    }
}

pub(crate) async fn dispatch_activity(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: ActivityCommand,
) -> Result<()> {
    match command {
        ActivityCommand::States {
            user_id,
            max_users,
            max_rallies,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(max_users > 0, "--max-users must be greater than zero");
            ensure!(max_rallies > 0, "--max-rallies must be greater than zero");
            let pool = connect_database(database_url, config).await?;
            activity_states(&pool, max_users, max_rallies).await
        }
        ActivityCommand::RallyViewState {
            user_id,
            rally_result_id,
            require_user_result,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                rally_result_id > 0,
                "--rally-result-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            rally_view_state(&pool, user_id, rally_result_id, require_user_result).await
        }
    }
}

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

pub(crate) async fn dispatch_program(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: ProgramCommand,
) -> Result<()> {
    match command {
        ProgramCommand::Verify { program_source_id } => {
            let pool = connect_database(database_url, config).await?;
            verify_program(&pool, program_source_id).await
        }
        ProgramCommand::VerifySource { source_file } => verify_source_file(&source_file),
        ProgramCommand::SimulateSource {
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
        ProgramCommand::CreateSource {
            user_id,
            source_name,
            source_code,
        } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
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
        ProgramCommand::UpdateSource {
            user_id,
            program_source_id,
            source_name,
            source_code,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                program_source_id > 0,
                "--program-source-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
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
        ProgramCommand::DeleteSource {
            user_id,
            program_source_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                program_source_id > 0,
                "--program-source-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            delete_program_source(&pool, user_id, program_source_id).await
        }
        ProgramCommand::SourceStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            program_source_states(&pool, user_id).await
        }
    }
}

pub(crate) async fn dispatch_user(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: UserCommand,
) -> Result<()> {
    match command {
        UserCommand::AccountState { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            account_state(&pool, user_id).await
        }
        UserCommand::Create {
            username,
            email,
            password,
        } => {
            ensure!(!username.is_empty(), "--username must not be empty");
            ensure!(!email.is_empty(), "--email must not be empty");
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(database_url, config).await?;
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
        UserCommand::UpdateAccount {
            user_id,
            username,
            email,
            password,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(!username.is_empty(), "--username must not be empty");
            ensure!(!email.is_empty(), "--email must not be empty");
            if let Some(password) = &password {
                ensure!(!password.is_empty(), "--password must not be empty");
            }
            let pool = connect_database(database_url, config).await?;
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
        UserCommand::VerifyLogin {
            login_name,
            password,
        } => {
            ensure!(!login_name.is_empty(), "--login-name must not be empty");
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(database_url, config).await?;
            verify_login(
                &pool,
                robominer_db::VerifyLoginRequest {
                    login_name,
                    password,
                },
            )
            .await
        }
        UserCommand::VerifyPassword { user_id, password } => {
            ensure_positive_user_id(user_id)?;
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(database_url, config).await?;
            verify_user_password(
                &pool,
                robominer_db::VerifyUserPasswordRequest { user_id, password },
            )
            .await
        }
    }
}

pub(crate) async fn dispatch_achievement(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: AchievementCommand,
) -> Result<()> {
    match command {
        AchievementCommand::ClaimStep {
            user_id,
            achievement_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                achievement_id > 0,
                "--achievement-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            claim_achievement_step(
                &pool,
                robominer_db::ClaimAchievementStepRequest {
                    user_id,
                    achievement_id,
                },
            )
            .await
        }
        AchievementCommand::States { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            achievement_states(&pool, user_id).await
        }
        AchievementCommand::PageStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            achievement_page_states(&pool, user_id).await
        }
    }
}

pub(crate) async fn dispatch_rally(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: RallyCommand,
) -> Result<()> {
    match command {
        RallyCommand::Run {
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

            let pool = connect_database(database_url, config).await?;
            run_rally(&pool, options).await.map(|_| ())
        }
        RallyCommand::Pool {
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

            let pool = connect_database(database_url, config).await?;
            run_pool(&pool, options).await.map(|_| ())
        }
        RallyCommand::Rallies {
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

            let pool = connect_database(database_url, config).await?;
            run_rallies(&pool, options).await
        }
    }
}

pub(crate) async fn dispatch_migrate(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: MigrateCommand,
) -> Result<()> {
    match command {
        MigrateCommand::Apply => {
            let pool = connect_database(database_url, config).await?;
            migrate(&pool).await
        }
        MigrateCommand::Status { check } => {
            let pool = connect_database(database_url, config).await?;
            migrate_status(&pool, check).await
        }
    }
}

pub(crate) async fn dispatch_leaderboard(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: LeaderboardCommand,
) -> Result<()> {
    match command {
        LeaderboardCommand::States { max_entries } => {
            ensure!(max_entries > 0, "--max-entries must be greater than zero");
            let pool = connect_database(database_url, config).await?;
            leaderboard_states(&pool, max_entries).await
        }
    }
}

pub(crate) async fn dispatch_assets(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: AssetsCommand,
) -> Result<()> {
    match command {
        AssetsCommand::OreStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            user_ore_asset_states(&pool, user_id).await
        }
    }
}
