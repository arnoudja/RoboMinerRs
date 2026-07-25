use std::path::PathBuf;

use anyhow::Result;

use crate::cli::RallyCommand;
use crate::database::connect_database;
use crate::rally::{
    RunPoolOptions, RunRalliesOptions, RunRallyOptions, run_pool, run_rallies, run_rally,
    validate_run_pool_options, validate_run_rallies_options, validate_run_rally_options,
};

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
