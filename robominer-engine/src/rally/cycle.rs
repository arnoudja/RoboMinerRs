use anyhow::{Context, Result, ensure};

use super::run_single::{RunRallyOptions, run_rally};

pub(crate) struct RunRalliesOptions {
    pub(crate) once: bool,
    pub(crate) loop_mode: bool,
    pub(crate) sleep_seconds: u64,
    pub(crate) seed: u64,
    pub(crate) persist: bool,
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
            let summary = run_rallies_cycle(pool, &options, cycle).await?;
            tracing::info!(
                cycle,
                ran = summary.ran,
                skipped = summary.skipped,
                failed = summary.failed,
                persist = options.persist,
                "Completed rally poll cycle"
            );
            println!(
                "Completed rally poll cycle {cycle}: ran={} skipped={} failed={} persist={}",
                summary.ran, summary.skipped, summary.failed, options.persist
            );

            if shutdown.requested() {
                println!("Shutdown requested; exiting after completed rally poll cycle {cycle}");
                break;
            }

            let sleep_seconds = next_poll_sleep_seconds(pool, options.sleep_seconds).await?;
            tracing::info!(cycle, sleep_seconds, "Sleeping until next rally poll cycle");

            tokio::select! {
                _ = shutdown.wait() => {
                    println!("Shutdown requested; exiting before next rally poll cycle");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(sleep_seconds)) => {}
            }
        }

        return Ok(());
    }

    let summary = run_rallies_cycle(pool, &options, 0).await?;
    println!(
        "Processed mining areas: ran={} skipped={} failed={} persist={}",
        summary.ran, summary.skipped, summary.failed, options.persist
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
    let mut failed = 0;

    for mining_area in mining_areas {
        match run_rally(
            pool,
            RunRallyOptions {
                mining_area_id: mining_area.id,
                seed: options
                    .seed
                    .wrapping_add(cycle)
                    .wrapping_add(mining_area.id as u64),
                persist: options.persist,
                result_data_file: None,
                quiet_when_empty: true,
            },
        )
        .await
        {
            Ok(true) => {
                tracing::info!(mining_area_id = mining_area.id, cycle, "rally completed");
                ran += 1;
            }
            Ok(false) => {
                skipped += 1;
            }
            Err(error) => {
                tracing::error!(
                    mining_area_id = mining_area.id,
                    cycle,
                    error = %error,
                    "rally area failed; continuing cycle"
                );
                eprintln!("Failed mining area {}: {error:#}", mining_area.id);
                failed += 1;
            }
        }
    }

    if options.persist {
        match crate::mining::claim_all_ready_results(pool).await {
            Ok(summary) if summary.users_processed > 0 => {
                tracing::info!(
                    cycle,
                    users = summary.users_processed,
                    queues = summary.total_queues_claimed,
                    "Completed wallet claim pass"
                );
                println!(
                    "Wallet claim pass: users={} queues={}",
                    summary.users_processed, summary.total_queues_claimed
                );
            }
            Ok(_) => {}
            Err(error) => {
                tracing::error!(cycle, error = %error, "Wallet claim pass failed");
                eprintln!("Wallet claim pass failed: {error:#}");
            }
        }
    }

    Ok(RunRalliesSummary {
        ran,
        skipped,
        failed,
    })
}

pub(crate) fn validate_run_rallies_options(options: &RunRalliesOptions) -> Result<()> {
    ensure!(
        options.once ^ options.loop_mode,
        "rally rallies requires exactly one of --once or --loop"
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

/// Max poll interval, shortened when the next claimable rally or wallet claim is sooner.
async fn next_poll_sleep_seconds(
    pool: &robominer_db::MySqlPool,
    max_sleep_seconds: u64,
) -> Result<u64> {
    let candidates = robominer_db::list_next_claim_rally_candidates(pool)
        .await
        .context("failed to load next claim rally candidates")?;
    let rally_delay = robominer_domain::next_claimable_rally_delay_seconds(&candidates)
        .unwrap_or(max_sleep_seconds);
    let wallet_delay = robominer_db::next_wallet_claim_delay_seconds(pool, max_sleep_seconds)
        .await
        .context("failed to load next wallet claim delay")?;
    Ok(rally_delay.min(wallet_delay).min(max_sleep_seconds))
}

struct RunRalliesSummary {
    ran: usize,
    skipped: usize,
    failed: usize,
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

