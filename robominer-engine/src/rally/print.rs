pub(super) fn print_rally_summary(
    loadout: &robominer_domain::loadout::RallyLoadout,
    outcome: &robominer_domain::simulation::RallyOutcome,
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

pub(super) fn print_pool_summary(
    loadout: &robominer_domain::loadout::PoolLoadout,
    outcome: &robominer_domain::simulation::PoolRallyOutcome,
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

fn pool_ore_summary(ore_results: &[robominer_domain::simulation::PoolItemOreOutcome]) -> String {
    if ore_results.is_empty() {
        return "none".to_string();
    }

    ore_results
        .iter()
        .map(|ore| format!("{}:{}", ore.ore_id, ore.amount))
        .collect::<Vec<_>>()
        .join(",")
}
