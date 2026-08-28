use anyhow::{Context, Result, ensure};
use std::collections::HashMap;

use crate::db_outcome::finish_db_outcome;
use crate::output::escape_state_field;

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
        let result = robominer_db::claim_user_results(pool, user_id)
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
        let mut shutdown = claim_shutdown_signal();

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

struct ClaimShutdownSignal {
    receiver: tokio::sync::watch::Receiver<bool>,
}

impl ClaimShutdownSignal {
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

fn claim_shutdown_signal() -> ClaimShutdownSignal {
    let (sender, receiver) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("failed to listen for shutdown signal: {error}");
            return;
        }

        let _ = sender.send(true);
    });

    ClaimShutdownSignal { receiver }
}

pub(crate) async fn claim_results(pool: &robominer_db::MySqlPool, user_id: i64) -> Result<()> {
    let result = robominer_db::claim_user_results(pool, user_id)
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

pub(crate) async fn enqueue_mining(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::EnqueueMiningRequest,
) -> Result<()> {
    let robot_id = request.robot_id;
    let mining_area_id = request.mining_area_id;
    finish_db_outcome(
        robominer_db::enqueue_mining(pool, request)
            .await
            .context("failed to enqueue mining run")?,
        |result| {
            println!(
                "Enqueued {} mining run(s) for robot {} in mining area {}",
                result.inserted_queues, robot_id, mining_area_id
            );
            Ok(())
        },
        |rejection| {
            format!(
                "unable to enqueue mining run: {}",
                robominer_domain::rejection_messages::enqueue_mining_rejection_cli_message(
                    rejection
                )
            )
        },
    )
}

pub(crate) async fn cancel_mining_queue(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::CancelMiningQueueRequest,
) -> Result<()> {
    finish_db_outcome(
        robominer_db::cancel_mining_queue(pool, request)
            .await
            .context("failed to cancel mining queue item")?,
        |result| {
            println!("Canceled mining queue {}", result.mining_queue_id);
            Ok(())
        },
        |rejection| {
            format!(
                "unable to cancel mining queue item: {}",
                robominer_domain::rejection_messages::cancel_mining_queue_rejection_cli_message(
                    rejection
                )
            )
        },
    )
}

pub(crate) async fn mining_queue_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<()> {
    let states = robominer_db::list_mining_queue_states_for_user(pool, user_id)
        .await
        .context("failed to load mining queue states")?;

    for state in states {
        println!(
            "{}\t{}\t{}\t{}",
            state.mining_queue_id,
            state.robot_id,
            state.status.as_str(),
            state.time_left_seconds
        );
    }

    Ok(())
}

pub(crate) async fn mining_queue_page_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<()> {
    let robots = robominer_db::list_mining_queue_page_robots(pool, user_id)
        .await
        .context("failed to load mining queue page robots")?;
    let areas = robominer_db::list_mining_queue_page_areas(pool, user_id)
        .await
        .context("failed to load mining queue page areas")?;
    let area_costs = robominer_db::list_mining_queue_page_area_costs(pool, user_id)
        .await
        .context("failed to load mining queue page area costs")?;
    let area_supplies = robominer_db::list_mining_queue_page_area_supplies(pool, user_id)
        .await
        .context("failed to load mining queue page area supplies")?;
    let area_yields = robominer_db::list_mining_queue_page_area_yields(pool, user_id)
        .await
        .context("failed to load mining queue page area yields")?;
    let queue_items = robominer_db::list_mining_queue_page_items(pool, user_id)
        .await
        .context("failed to load mining queue page items")?;
    let queue_states = robominer_db::list_mining_queue_states_for_user(pool, user_id)
        .await
        .context("failed to load mining queue states")?;
    let scores = robominer_db::list_robot_mining_area_scores_for_user(pool, user_id)
        .await
        .context("failed to load mining area scores")?;

    let queue_state_map: HashMap<i64, robominer_db::MiningQueueStateRecord> = queue_states
        .into_iter()
        .map(|state| (state.mining_queue_id, state))
        .collect();

    for robot in robots {
        println!(
            "R\t{}\t{}",
            robot.robot_id,
            escape_state_field(&robot.robot_name)
        );
    }

    for area in areas {
        println!(
            "A\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            area.mining_area_id,
            escape_state_field(&area.area_name),
            area.tax_rate,
            area.depot_tax_rate,
            area.mining_time,
            area.max_moves,
            area.size_x,
            area.size_y
        );
    }

    for cost in area_costs {
        println!(
            "C\t{}\t{}\t{}\t{}",
            cost.mining_area_id,
            cost.ore_id,
            escape_state_field(&cost.ore_name),
            cost.amount
        );
    }

    for supply in area_supplies {
        println!(
            "S\t{}\t{}\t{}\t{}\t{}",
            supply.mining_area_id,
            supply.ore_id,
            escape_state_field(&supply.ore_name),
            supply.supply,
            supply.radius
        );
    }

    for area_yield in area_yields {
        println!(
            "H\t{}\t{}\t{}\t{}",
            area_yield.mining_area_id,
            area_yield.ore_id,
            escape_state_field(&area_yield.ore_name),
            area_yield.percentage
        );
    }

    for queue_item in queue_items {
        if let Some(state) = queue_state_map.get(&queue_item.mining_queue_id) {
            println!(
                "Q\t{}\t{}\t{}\t{}\t{}\t{}",
                queue_item.mining_queue_id,
                queue_item.robot_id,
                queue_item.mining_area_id,
                escape_state_field(&queue_item.area_name),
                state.status.as_str(),
                state.time_left_seconds
            );
        }
    }

    for score in scores {
        println!(
            "P\t{}\t{}\t{}",
            score.robot_id, score.mining_area_id, score.score
        );
    }

    Ok(())
}

pub(crate) async fn mining_area_scores(pool: &robominer_db::MySqlPool, user_id: i64) -> Result<()> {
    let scores = robominer_db::list_robot_mining_area_scores_for_user(pool, user_id)
        .await
        .context("failed to load robot mining area scores")?;

    for score in scores {
        println!(
            "{}\t{}\t{}",
            score.robot_id, score.mining_area_id, score.score
        );
    }

    Ok(())
}

pub(crate) async fn mining_result_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    max_results: i64,
) -> Result<()> {
    let robots = robominer_db::list_mining_queue_page_robots(pool, user_id)
        .await
        .context("failed to load mining result robots")?;
    let results = robominer_db::list_mining_result_states_for_user(pool, user_id, max_results)
        .await
        .context("failed to load mining result states")?;
    let ore_results =
        robominer_db::list_mining_result_ore_states_for_user(pool, user_id, max_results)
            .await
            .context("failed to load mining result ore states")?;
    let action_results =
        robominer_db::list_mining_result_action_states_for_user(pool, user_id, max_results)
            .await
            .context("failed to load mining result action states")?;

    for robot in robots {
        println!(
            "R\t{}\t{}",
            robot.robot_id,
            escape_state_field(&robot.robot_name)
        );
    }

    for result in results {
        println!(
            "Q\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            result.robot_id,
            result.mining_queue_id,
            escape_state_field(&result.mining_area_name),
            result
                .rally_result_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            result.score,
            result.total_ore_mined,
            result.total_tax,
            result.total_reward,
            result.creation_time_millis,
            result.mining_end_time_millis
        );
    }

    for ore_result in ore_results {
        println!(
            "O\t{}\t{}\t{}\t{}\t{}\t{}",
            ore_result.mining_queue_id,
            ore_result.ore_id,
            escape_state_field(&ore_result.ore_name),
            ore_result.amount,
            ore_result.tax,
            ore_result.reward
        );
    }

    for action_result in action_results {
        println!(
            "D\t{}\t{}\t{}",
            action_result.mining_queue_id, action_result.action_type, action_result.amount
        );
    }

    Ok(())
}

pub(crate) async fn mining_area_overview_states(pool: &robominer_db::MySqlPool) -> Result<()> {
    let ores = robominer_db::list_mining_area_overview_ores(pool)
        .await
        .context("failed to load mining area overview ores")?;
    let areas = robominer_db::list_mining_area_overview_areas(pool)
        .await
        .context("failed to load mining area overview areas")?;
    let ore_averages = robominer_db::list_mining_area_overview_ore_averages(pool)
        .await
        .context("failed to load mining area overview ore averages")?;

    for ore in ores {
        println!("O\t{}\t{}", ore.ore_id, escape_state_field(&ore.ore_name));
    }

    for area in areas {
        println!(
            "A\t{}\t{}\t{}",
            area.mining_area_id,
            escape_state_field(&area.area_name),
            area.total_average_ore_per_run
        );
    }

    for average in ore_averages {
        println!(
            "P\t{}\t{}\t{}",
            average.mining_area_id, average.ore_id, average.average_ore_per_run
        );
    }

    Ok(())
}
