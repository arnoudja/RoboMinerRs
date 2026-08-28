use anyhow::{Context, Result, ensure};
use std::fs;
use std::path::PathBuf;

pub(crate) struct RunRallyOptions {
    pub(crate) mining_area_id: i64,
    pub(crate) seed: u64,
    pub(crate) persist: bool,
    pub(crate) result_data_file: Option<PathBuf>,
    /// When true, skip logging for areas with no ready rally (poll loop).
    pub(crate) quiet_when_empty: bool,
}

pub(crate) struct RunPoolOptions {
    pub(crate) pool_id: i64,
    pub(crate) seed: u64,
    pub(crate) persist: bool,
    pub(crate) until_complete: bool,
    pub(crate) max_rallies: u64,
}

pub(crate) async fn run_rally(
    pool: &robominer_db::MySqlPool,
    options: RunRallyOptions,
) -> Result<bool> {
    let loadout = robominer_domain::load_next_rally_loadout_with_claim(
        pool,
        options.mining_area_id,
        options.persist,
    )
    .await
    .with_context(|| {
        format!(
            "failed to load next rally for mining area {}",
            options.mining_area_id
        )
    })?;
    let Some(loadout) = loadout else {
        if !options.quiet_when_empty {
            println!("No ready rally for mining area {}", options.mining_area_id);
        }
        return Ok(false);
    };

    let mining_area_id = options.mining_area_id;
    let seed = options.seed;
    let sim_loadout = loadout.clone();
    let run = tokio::task::spawn_blocking(move || {
        robominer_domain::run_rally_loadout_with_animation_seed(&sim_loadout, seed)
    })
    .await
    .context("rally simulation task failed")?
    .with_context(|| format!("failed to run rally for mining area {mining_area_id}"))?;
    let outcome = &run.outcome;

    crate::rally::print::print_rally_summary(&loadout, outcome);

    if options.persist {
        let result_data = match options.result_data_file {
            Some(result_data_file) => tokio::task::spawn_blocking(move || {
                fs::read_to_string(&result_data_file).with_context(|| {
                    format!(
                        "failed to read result data file {}",
                        result_data_file.display()
                    )
                })
            })
            .await
            .context("result data file read task failed")??,
            None => run.result_data,
        };
        let persist_outcome =
            robominer_domain::persist_rally_outcome(pool, &loadout, outcome, &result_data)
                .await
                .context("failed to persist rally outcome")?;

        match persist_outcome {
            robominer_db::DbOutcome::Success(rally_result_id) => {
                tracing::info!(
                    mining_area_id,
                    rally_result_id,
                    queue_len = loadout.queue_entries.len(),
                    "Persisted rally result"
                );
                println!("Persisted rally result {rally_result_id}");
            }
            robominer_db::DbOutcome::Rejected(rejection) => {
                tracing::warn!(
                    mining_area_id,
                    ?rejection,
                    "Skipped persist: queue already finished by another worker"
                );
                println!(
                    "Skipped persist for mining area {mining_area_id}: queue already finished"
                );
                return Ok(false);
            }
        }
    } else {
        println!("Dry run: no database writes performed");
    }

    Ok(true)
}

pub(crate) async fn run_pool(
    pool: &robominer_db::MySqlPool,
    options: RunPoolOptions,
) -> Result<bool> {
    validate_run_pool_options(&options)?;

    if options.until_complete {
        return run_pool_until_complete(pool, options).await;
    }

    run_pool_once(pool, &options, 0).await
}

async fn run_pool_until_complete(
    pool: &robominer_db::MySqlPool,
    options: RunPoolOptions,
) -> Result<bool> {
    let mut ran = 0_u64;

    while ran < options.max_rallies {
        let did_run = run_pool_once(pool, &options, ran).await?;
        if !did_run {
            println!("Pool repeat complete: ran={ran}");
            return Ok(ran > 0);
        }

        ran += 1;
    }

    println!(
        "Pool repeat stopped after max-rallies {}: ran={ran}",
        options.max_rallies
    );

    Ok(ran > 0)
}

async fn run_pool_once(
    pool: &robominer_db::MySqlPool,
    options: &RunPoolOptions,
    rally_index: u64,
) -> Result<bool> {
    let loadout = robominer_domain::load_next_pool_rally_loadout(pool, options.pool_id)
        .await
        .with_context(|| {
            format!(
                "failed to load next pool rally for pool {}",
                options.pool_id
            )
        })?;
    let Some(loadout) = loadout else {
        println!("Pool {} not found", options.pool_id);
        return Ok(false);
    };

    if loadout.items.is_empty() {
        println!("No pool items for pool {}", options.pool_id);
        return Ok(false);
    }

    if loadout.is_complete() {
        println!("Pool {} is complete", options.pool_id);
        return Ok(false);
    }

    let pool_id = options.pool_id;
    let seed = options.seed.wrapping_add(rally_index);
    let sim_loadout = loadout.clone();
    let outcome = tokio::task::spawn_blocking(move || {
        robominer_domain::run_pool_loadout_with_seed(&sim_loadout, seed)
    })
    .await
    .context("pool simulation task failed")?
    .with_context(|| format!("failed to run pool rally for pool {pool_id}"))?;

    crate::rally::print::print_pool_summary(&loadout, &outcome);

    if options.persist {
        robominer_domain::persist_pool_rally_outcome(pool, &loadout, &outcome)
            .await
            .context("failed to persist pool rally outcome")?;

        println!("Persisted pool rally");
    } else {
        println!("Dry run: no database writes performed");
    }

    Ok(true)
}

pub(crate) fn validate_run_pool_options(options: &RunPoolOptions) -> Result<()> {
    ensure!(options.pool_id > 0, "--pool-id must be greater than zero");
    ensure!(
        options.max_rallies > 0,
        "--max-rallies must be greater than zero"
    );
    ensure!(
        !options.until_complete || options.persist,
        "--until-complete requires --persist so repeated pool runs can advance"
    );

    Ok(())
}

