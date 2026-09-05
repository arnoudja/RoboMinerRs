use anyhow::{Context, Result, ensure};

use crate::output::escape_state_field;
use crate::shutdown::shutdown_signal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClaimAllSummary {
    pub users_processed: usize,
    pub total_queues_claimed: u64,
}

pub(crate) struct RunClaimAllOptions {
    pub once: bool,
    pub loop_mode: bool,
    pub sleep_seconds: u64,
}

pub(crate) async fn claim_all_ready_results(
    pool: &robominer_db::MySqlPool,
) -> Result<ClaimAllSummary> {
    let user_ids = robominer_db::list_user_ids_with_claimable_mining_queues(pool)
        .await
        .context("failed to list users with claimable mining results")?;

    let mut summary = ClaimAllSummary {
        users_processed: 0,
        total_queues_claimed: 0,
    };

    for user_id in user_ids {
        let result = robominer_db::rally::claim_user_results(pool, user_id)
            .await
            .with_context(|| format!("failed to claim mining results for user {user_id}"))?;
        if result.claimed_queues > 0 {
            summary.users_processed += 1;
            summary.total_queues_claimed += result.claimed_queues;
            println!(
                "Claimed {} mining result(s) for user {user_id}",
                result.claimed_queues
            );
            for reward in &result.ore_rewards {
                println!(
                    "Added to wallet: {} +{}",
                    escape_state_field(&reward.ore_name),
                    reward.reward
                );
            }
        }
    }

    Ok(summary)
}

pub(crate) async fn run_claim_all(
    pool: &robominer_db::MySqlPool,
    options: RunClaimAllOptions,
) -> Result<()> {
    validate_run_claim_all_options(&options)?;

    if options.loop_mode {
        let mut cycle = 0_u64;
        let mut shutdown = shutdown_signal();

        loop {
            cycle += 1;
            let summary = claim_all_ready_results(pool).await?;
            tracing::info!(
                cycle,
                users = summary.users_processed,
                queues = summary.total_queues_claimed,
                "Completed wallet claim poll cycle"
            );
            println!(
                "Completed wallet claim poll cycle {cycle}: users={} queues={}",
                summary.users_processed, summary.total_queues_claimed
            );

            if shutdown.requested() {
                println!(
                    "Shutdown requested; exiting after completed wallet claim poll cycle {cycle}"
                );
                break;
            }

            let sleep_seconds =
                robominer_db::next_wallet_claim_delay_seconds(pool, options.sleep_seconds)
                    .await
                    .context("failed to compute next wallet claim delay")?;
            tracing::info!(
                cycle,
                sleep_seconds,
                "Sleeping until next wallet claim poll cycle"
            );

            tokio::select! {
                _ = shutdown.wait() => {
                    println!("Shutdown requested; exiting before next wallet claim poll cycle");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(sleep_seconds)) => {}
            }
        }

        return Ok(());
    }

    let summary = claim_all_ready_results(pool).await?;
    println!(
        "Processed wallet claims: users={} queues={}",
        summary.users_processed, summary.total_queues_claimed
    );

    Ok(())
}

pub(crate) fn validate_run_claim_all_options(options: &RunClaimAllOptions) -> Result<()> {
    ensure!(
        options.once ^ options.loop_mode,
        "mining claim-all requires exactly one of --once or --loop"
    );
    ensure!(
        options.sleep_seconds > 0,
        "--sleep-seconds must be greater than zero"
    );

    Ok(())
}

pub(crate) async fn claim_results(pool: &robominer_db::MySqlPool, user_id: i64) -> Result<()> {
    let result = robominer_db::rally::claim_user_results(pool, user_id)
        .await
        .with_context(|| format!("failed to claim mining results for user {user_id}"))?;

    println!(
        "Claimed {} mining result(s) for user {user_id}",
        result.claimed_queues
    );
    for reward in &result.ore_rewards {
        println!(
            "Added to wallet: {} +{}",
            escape_state_field(&reward.ore_name),
            reward.reward
        );
    }

    Ok(())
}
