use robominer_db::MySqlPool;
use robominer_test_support::{
    insert_area_supply, insert_committed_pending_changes, insert_finished_queue,
    insert_mining_area, insert_ore, insert_ore_price, insert_ore_result, insert_robot,
    insert_unfinished_queue, insert_user, insert_user_ore_asset,
    unique_prefix as shared_unique_prefix,
};

pub struct ClaimScenario {
    pub name: &'static str,
    pub user_id: i64,
    pub robot_id: i64,
    pub ai_robot_id: i64,
    pub mining_area_id: i64,
    pub ore_ids: Vec<i64>,
    pub ore_price_id: i64,
    pub queue_ids: Vec<i64>,
}

pub fn scenario(name: &str) {
    match name {
        "single_queue_tax25"
        | "dual_queue_batch_claim"
        | "skips_unfinished_queue"
        | "claim_cap_limited"
        | "claim_zero_tax"
        | "claim_multiple_ore_types" => {}
        other => panic!("unknown claim golden scenario: {other}"),
    }
}

pub async fn setup(pool: &MySqlPool, name: &str) -> ClaimScenario {
    match name {
        "single_queue_tax25" => single_queue_tax25(pool).await,
        "dual_queue_batch_claim" => dual_queue_batch_claim(pool).await,
        "skips_unfinished_queue" => skips_unfinished_queue(pool).await,
        "claim_cap_limited" => claim_cap_limited(pool).await,
        "claim_zero_tax" => claim_zero_tax(pool).await,
        "claim_multiple_ore_types" => claim_multiple_ore_types(pool).await,
        other => panic!("unknown claim golden scenario: {other}"),
    }
}

async fn single_queue_tax25(pool: &MySqlPool) -> ClaimScenario {
    let prefix = unique_prefix("single_queue_tax25");
    let primary_ore_id = insert_ore(pool, &format!("{prefix}-primary")).await;
    let secondary_ore_id = insert_ore(pool, &format!("{prefix}-secondary")).await;
    let ore_price_id = insert_ore_price(pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(pool, &prefix).await;
    let ai_robot_id = insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(pool, &prefix, ore_price_id, ai_robot_id, 25).await;

    insert_area_supply(pool, mining_area_id, primary_ore_id, 10, 2).await;
    insert_area_supply(pool, mining_area_id, secondary_ore_id, 3, 1).await;

    let mining_queue_id = insert_finished_queue(pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result(pool, mining_queue_id, primary_ore_id, 10).await;
    insert_user_ore_asset(pool, user_id, primary_ore_id, 2, 8).await;
    insert_committed_pending_changes(pool, robot_id).await;

    ClaimScenario {
        name: "single_queue_tax25",
        user_id,
        robot_id,
        ai_robot_id,
        mining_area_id,
        ore_ids: vec![primary_ore_id, secondary_ore_id],
        ore_price_id,
        queue_ids: vec![mining_queue_id],
    }
}

async fn dual_queue_batch_claim(pool: &MySqlPool) -> ClaimScenario {
    let prefix = unique_prefix("dual_queue_batch_claim");
    let primary_ore_id = insert_ore(pool, &format!("{prefix}-primary")).await;
    let ore_price_id = insert_ore_price(pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(pool, &prefix).await;
    let ai_robot_id = insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(pool, &prefix, ore_price_id, ai_robot_id, 25).await;

    insert_area_supply(pool, mining_area_id, primary_ore_id, 10, 2).await;

    let first_queue_id = insert_finished_queue(pool, mining_area_id, robot_id, -40, -30).await;
    insert_ore_result(pool, first_queue_id, primary_ore_id, 10).await;
    let second_queue_id = insert_finished_queue(pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result(pool, second_queue_id, primary_ore_id, 4).await;
    insert_user_ore_asset(pool, user_id, primary_ore_id, 2, 25).await;

    ClaimScenario {
        name: "dual_queue_batch_claim",
        user_id,
        robot_id,
        ai_robot_id,
        mining_area_id,
        ore_ids: vec![primary_ore_id],
        ore_price_id,
        queue_ids: vec![first_queue_id, second_queue_id],
    }
}

async fn skips_unfinished_queue(pool: &MySqlPool) -> ClaimScenario {
    let prefix = unique_prefix("skips_unfinished_queue");
    let primary_ore_id = insert_ore(pool, &format!("{prefix}-primary")).await;
    let ore_price_id = insert_ore_price(pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(pool, &prefix).await;
    let ai_robot_id = insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(pool, &prefix, ore_price_id, ai_robot_id, 25).await;

    insert_area_supply(pool, mining_area_id, primary_ore_id, 10, 2).await;

    let finished_queue_id = insert_finished_queue(pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result(pool, finished_queue_id, primary_ore_id, 6).await;
    let unfinished_queue_id = insert_unfinished_queue(pool, mining_area_id, robot_id).await;
    insert_user_ore_asset(pool, user_id, primary_ore_id, 0, 10).await;

    ClaimScenario {
        name: "skips_unfinished_queue",
        user_id,
        robot_id,
        ai_robot_id,
        mining_area_id,
        ore_ids: vec![primary_ore_id],
        ore_price_id,
        queue_ids: vec![finished_queue_id, unfinished_queue_id],
    }
}

async fn claim_cap_limited(pool: &MySqlPool) -> ClaimScenario {
    let prefix = unique_prefix("claim_cap_limited");
    let primary_ore_id = insert_ore(pool, &format!("{prefix}-primary")).await;
    let ore_price_id = insert_ore_price(pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(pool, &prefix).await;
    let ai_robot_id = insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(pool, &prefix, ore_price_id, ai_robot_id, 25).await;

    insert_area_supply(pool, mining_area_id, primary_ore_id, 10, 2).await;

    let mining_queue_id = insert_finished_queue(pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result(pool, mining_queue_id, primary_ore_id, 12).await;
    insert_user_ore_asset(pool, user_id, primary_ore_id, 22, 25).await;

    ClaimScenario {
        name: "claim_cap_limited",
        user_id,
        robot_id,
        ai_robot_id,
        mining_area_id,
        ore_ids: vec![primary_ore_id],
        ore_price_id,
        queue_ids: vec![mining_queue_id],
    }
}

async fn claim_zero_tax(pool: &MySqlPool) -> ClaimScenario {
    let prefix = unique_prefix("claim_zero_tax");
    let primary_ore_id = insert_ore(pool, &format!("{prefix}-primary")).await;
    let ore_price_id = insert_ore_price(pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(pool, &prefix).await;
    let ai_robot_id = insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(pool, &prefix, ore_price_id, ai_robot_id, 0).await;

    insert_area_supply(pool, mining_area_id, primary_ore_id, 10, 2).await;

    let mining_queue_id = insert_finished_queue(pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result(pool, mining_queue_id, primary_ore_id, 15).await;
    insert_user_ore_asset(pool, user_id, primary_ore_id, 0, 25).await;

    ClaimScenario {
        name: "claim_zero_tax",
        user_id,
        robot_id,
        ai_robot_id,
        mining_area_id,
        ore_ids: vec![primary_ore_id],
        ore_price_id,
        queue_ids: vec![mining_queue_id],
    }
}

async fn claim_multiple_ore_types(pool: &MySqlPool) -> ClaimScenario {
    let prefix = unique_prefix("claim_multiple_ore_types");
    let primary_ore_id = insert_ore(pool, &format!("{prefix}-primary")).await;
    let secondary_ore_id = insert_ore(pool, &format!("{prefix}-secondary")).await;
    let ore_price_id = insert_ore_price(pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(pool, &prefix).await;
    let ai_robot_id = insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(pool, &prefix, ore_price_id, ai_robot_id, 25).await;

    insert_area_supply(pool, mining_area_id, primary_ore_id, 10, 2).await;
    insert_area_supply(pool, mining_area_id, secondary_ore_id, 5, 1).await;

    let mining_queue_id = insert_finished_queue(pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result(pool, mining_queue_id, primary_ore_id, 10).await;
    insert_ore_result(pool, mining_queue_id, secondary_ore_id, 4).await;
    insert_user_ore_asset(pool, user_id, primary_ore_id, 1, 25).await;
    insert_user_ore_asset(pool, user_id, secondary_ore_id, 0, 25).await;

    ClaimScenario {
        name: "claim_multiple_ore_types",
        user_id,
        robot_id,
        ai_robot_id,
        mining_area_id,
        ore_ids: vec![primary_ore_id, secondary_ore_id],
        ore_price_id,
        queue_ids: vec![mining_queue_id],
    }
}

pub async fn cleanup(pool: &MySqlPool, scenario: &ClaimScenario) {
    for queue_id in &scenario.queue_ids {
        let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
            .bind(queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM RobotActionsDone WHERE miningQueueId = ?")
            .bind(queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .execute(pool)
            .await;
    }

    let _ = sqlx::query("DELETE FROM PendingRobotChanges WHERE robotId = ?")
        .bind(scenario.robot_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM RobotLifetimeResult WHERE robotId = ?")
        .bind(scenario.robot_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningAreaLifetimeResult WHERE miningAreaId = ?")
        .bind(scenario.mining_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
        .bind(scenario.user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId = ?")
        .bind(scenario.mining_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(scenario.mining_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id IN (?, ?)")
        .bind(scenario.ai_robot_id)
        .bind(scenario.robot_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(scenario.user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(scenario.ore_price_id)
        .execute(pool)
        .await;
    for ore_id in &scenario.ore_ids {
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(ore_id)
            .execute(pool)
            .await;
    }
}

fn unique_prefix(scenario: &str) -> String {
    shared_unique_prefix(&format!("golden-claim-{scenario}"))
}

pub fn ore_index(scenario: &ClaimScenario, ore_id: i64) -> usize {
    scenario
        .ore_ids
        .iter()
        .position(|candidate| *candidate == ore_id)
        .unwrap_or_else(|| panic!("unknown ore id {ore_id} for scenario {}", scenario.name))
}

pub fn queue_index(scenario: &ClaimScenario, queue_id: i64) -> usize {
    scenario
        .queue_ids
        .iter()
        .position(|candidate| *candidate == queue_id)
        .unwrap_or_else(|| panic!("unknown queue id {queue_id} for scenario {}", scenario.name))
}
