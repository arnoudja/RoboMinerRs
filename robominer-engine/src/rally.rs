use anyhow::{Context, Result, ensure};
use std::fs;
use std::path::PathBuf;

pub(crate) struct RunRallyOptions {
    pub(crate) mining_area_id: i64,
    pub(crate) seed: u64,
    pub(crate) persist: bool,
    pub(crate) result_data_file: Option<PathBuf>,
}

pub(crate) struct RunPoolOptions {
    pub(crate) pool_id: i64,
    pub(crate) seed: u64,
    pub(crate) persist: bool,
    pub(crate) until_complete: bool,
    pub(crate) max_rallies: u64,
}

pub(crate) struct RunRalliesOptions {
    pub(crate) once: bool,
    pub(crate) loop_mode: bool,
    pub(crate) sleep_seconds: u64,
    pub(crate) seed: u64,
    pub(crate) persist: bool,
}

pub(crate) async fn run_rally(
    pool: &robominer_db::MySqlPool,
    options: RunRallyOptions,
) -> Result<bool> {
    validate_run_rally_options(&options)?;

    let loadout = robominer_domain::load_next_rally_loadout(pool, options.mining_area_id)
        .await
        .with_context(|| {
            format!(
                "failed to load next rally for mining area {}",
                options.mining_area_id
            )
        })?;
    let Some(loadout) = loadout else {
        println!("No ready rally for mining area {}", options.mining_area_id);
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

    print_rally_summary(&loadout, outcome);

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
        let rally_result_id =
            robominer_domain::persist_rally_outcome(pool, &loadout, outcome, &result_data)
                .await
                .context("failed to persist rally outcome")?;

        println!("Persisted rally result {rally_result_id}");
    } else {
        println!("Dry run: no database writes performed");
    }

    Ok(true)
}

pub(crate) fn validate_run_rally_options(options: &RunRallyOptions) -> Result<()> {
    let _ = options;

    Ok(())
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

    print_pool_summary(&loadout, &outcome);

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

pub(crate) async fn run_rallies(
    pool: &robominer_db::MySqlPool,
    options: RunRalliesOptions,
) -> Result<()> {
    validate_run_rallies_options(&options)?;

    if options.loop_mode {
        let mut cycle = 0_u64;
        let mut shutdown = shutdown_signal();

        loop {
            cycle += 1;
            println!("Starting rally poll cycle {cycle}");
            let summary = run_rallies_cycle(pool, &options, cycle).await?;
            println!(
                "Completed rally poll cycle {cycle}: ran={} skipped={} persist={}",
                summary.ran, summary.skipped, options.persist
            );

            if shutdown.requested() {
                println!("Shutdown requested; exiting after completed rally poll cycle {cycle}");
                break;
            }

            tokio::select! {
                _ = shutdown.wait() => {
                    println!("Shutdown requested; exiting before next rally poll cycle");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(options.sleep_seconds)) => {}
            }
        }

        return Ok(());
    }

    let summary = run_rallies_cycle(pool, &options, 0).await?;
    println!(
        "Processed mining areas: ran={} skipped={} persist={}",
        summary.ran, summary.skipped, options.persist
    );

    Ok(())
}

async fn run_rallies_cycle(
    pool: &robominer_db::MySqlPool,
    options: &RunRalliesOptions,
    cycle: u64,
) -> Result<RunRalliesSummary> {
    let mining_areas = robominer_db::list_mining_areas(pool)
        .await
        .context("failed to load mining areas")?;
    let mut ran = 0;
    let mut skipped = 0;

    println!("Processing {} mining areas", mining_areas.len());

    for mining_area in mining_areas {
        println!(
            "Processing mining area {} ({})",
            mining_area.id, mining_area.area_name
        );

        let did_run = run_rally(
            pool,
            RunRallyOptions {
                mining_area_id: mining_area.id,
                seed: options
                    .seed
                    .wrapping_add(cycle)
                    .wrapping_add(mining_area.id as u64),
                persist: options.persist,
                result_data_file: None,
            },
        )
        .await
        .with_context(|| format!("failed to process mining area {}", mining_area.id))?;

        if did_run {
            ran += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(RunRalliesSummary { ran, skipped })
}

pub(crate) fn validate_run_rallies_options(options: &RunRalliesOptions) -> Result<()> {
    ensure!(
        options.once ^ options.loop_mode,
        "run-rallies requires exactly one of --once or --loop"
    );
    ensure!(
        options.sleep_seconds > 0,
        "--sleep-seconds must be greater than zero"
    );
    ensure!(
        !options.loop_mode || options.persist,
        "--loop requires --persist so continuous polling cannot run as an accidental dry-run"
    );

    Ok(())
}

struct RunRalliesSummary {
    ran: usize,
    skipped: usize,
}

struct ShutdownSignal {
    receiver: tokio::sync::watch::Receiver<bool>,
}

impl ShutdownSignal {
    fn requested(&self) -> bool {
        *self.receiver.borrow()
    }

    async fn wait(&mut self) {
        if self.requested() {
            return;
        }

        let _ = self.receiver.changed().await;
    }
}

fn shutdown_signal() -> ShutdownSignal {
    let (sender, receiver) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("failed to listen for shutdown signal: {error}");
            return;
        }

        let _ = sender.send(true);
    });

    ShutdownSignal { receiver }
}

fn print_rally_summary(
    loadout: &robominer_domain::RallyLoadout,
    outcome: &robominer_domain::RallyOutcome,
) {
    println!("Rally complete");
    println!("mining area: {}", outcome.mining_area_id);
    println!("turns: {}", outcome.final_time);
    println!("queued robots: {}", loadout.queue_entries.len());
    println!("ai robots: {}", loadout.ai_robot_count());

    for participant in &outcome.participants {
        let queue_id = participant
            .queue_id
            .map(|queue_id| queue_id.to_string())
            .unwrap_or_else(|| "AI".to_string());
        let position = participant.position;

        println!(
            "player {} queue={} robot={}{} score={:.3} ore={:?} position=x={:.3} y={:.3} orientation={}",
            participant.player_number,
            queue_id,
            participant.robot_id,
            if participant.is_ai { " ai" } else { "" },
            participant.score,
            participant.ore,
            position.x,
            position.y,
            position.orientation
        );
        println!(
            "player {} actions: wait={} forward={} backward={} rotate_right={} rotate_left={} mine={} dump={}",
            participant.player_number,
            participant.actions_done[1],
            participant.actions_done[2],
            participant.actions_done[3],
            participant.actions_done[4],
            participant.actions_done[5],
            participant.actions_done[6],
            participant.actions_done[7]
        );
    }
}

fn print_pool_summary(
    loadout: &robominer_domain::PoolLoadout,
    outcome: &robominer_domain::PoolRallyOutcome,
) {
    println!("Pool rally complete");
    println!("pool: {}", outcome.pool_id);
    println!("mining area: {}", outcome.mining_area_id);
    println!("turns: {}", outcome.final_time);
    println!("pool items: {}", outcome.items.len());
    println!("required runs: {}", loadout.pool.required_runs);

    for item in &outcome.items {
        let runs_done = loadout
            .items
            .iter()
            .find(|loadout_item| loadout_item.item.id == item.pool_item_id)
            .map(|loadout_item| loadout_item.item.runs_done)
            .unwrap_or_default();

        println!(
            "player {} pool_item={} robot={} runs_done={} score={:.3} ore={}",
            item.player_number,
            item.pool_item_id,
            item.robot_id,
            runs_done,
            item.score,
            pool_ore_summary(&item.ore_results)
        );
    }
}

fn pool_ore_summary(ore_results: &[robominer_domain::PoolItemOreOutcome]) -> String {
    if ore_results.is_empty() {
        return "none".to_string();
    }

    ore_results
        .iter()
        .map(|ore| format!("{}:{}", ore.ore_id, ore.amount))
        .collect::<Vec<_>>()
        .join(",")
}
