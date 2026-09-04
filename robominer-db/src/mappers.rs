use crate::types::*;

pub(crate) type MiningAreaRow = (i64, String, i64, i32, i32, i32, i32, i32, i32, i32, i64);
pub(crate) type PoolRow = (i64, i64, i32);
pub(crate) type PoolItemRow = (i64, i64, i64, String, f64, i32);
pub(crate) type MiningRallyQueueRow = (
    i64,
    i64,
    i64,
    i64,
    Option<i64>,
    Option<i32>,
    Option<f64>,
    bool,
    i32,
);

pub(crate) fn mining_area_rows(rows: Vec<MiningAreaRow>) -> Vec<MiningAreaRecord> {
    rows.into_iter().map(mining_area_record).collect()
}

pub(crate) fn mining_area_record(
    (
        id,
        area_name,
        ore_price_id,
        size_x,
        size_y,
        max_moves,
        mining_time,
        tax_rate,
        depot_tax_rate,
        score_ore_target,
        ai_robot_id,
    ): MiningAreaRow,
) -> MiningAreaRecord {
    MiningAreaRecord {
        id,
        area_name,
        ore_price_id,
        size_x,
        size_y,
        max_moves,
        mining_time,
        tax_rate,
        depot_tax_rate,
        score_ore_target,
        ai_robot_id,
    }
}

pub(crate) fn mining_rally_queue_rows(
    rows: Vec<MiningRallyQueueRow>,
) -> Vec<MiningRallyQueueRecord> {
    let mut seen_users = Vec::new();

    rows.into_iter()
        .filter(|row| {
            if seen_users.contains(&row.3) {
                false
            } else {
                seen_users.push(row.3);
                true
            }
        })
        .take(4)
        .map(mining_rally_queue_record)
        .collect()
}

fn mining_rally_queue_record(
    (
        id,
        mining_area_id,
        robot_id,
        user_id,
        rally_result_id,
        player_number,
        score,
        claimed,
        seconds_left,
    ): MiningRallyQueueRow,
) -> MiningRallyQueueRecord {
    MiningRallyQueueRecord {
        queue: MiningQueueRecord {
            id,
            mining_area_id,
            robot_id,
            rally_result_id,
            player_number,
            score,
            claimed,
        },
        user_id,
        seconds_left,
    }
}

pub(crate) fn pool_record((id, mining_area_id, required_runs): PoolRow) -> PoolRecord {
    PoolRecord {
        id,
        mining_area_id,
        required_runs,
    }
}

pub(crate) fn pool_item_rows(rows: Vec<PoolItemRow>) -> Vec<PoolItemRecord> {
    rows.into_iter().map(pool_item_record).collect()
}

pub(crate) fn next_pool_rally_item_rows(rows: Vec<PoolItemRow>) -> Vec<PoolItemRecord> {
    let first_runs_done = rows.first().map(|row| row.5);

    rows.into_iter()
        .filter(|row| Some(row.5) == first_runs_done)
        .map(pool_item_record)
        .collect()
}

fn pool_item_record(
    (id, pool_id, robot_id, source_code, total_score, runs_done): PoolItemRow,
) -> PoolItemRecord {
    PoolItemRecord {
        id,
        pool_id,
        robot_id,
        source_code,
        total_score,
        runs_done,
    }
}
