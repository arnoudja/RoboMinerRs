use sqlx::MySqlPool;

use crate::{MiningQueueStateRecord, MiningQueueStatus};

pub async fn list_mining_queue_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueueStateRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64, i32, i32, i8, Option<i64>, Option<i64>, i64)>(
        "SELECT MiningQueue.id, MiningQueue.robotId, Robot.rechargeTime, MiningArea.miningTime, \
                CASE WHEN Robot.rechargeEndTime > NOW() \
                       AND (Robot.miningEndTime IS NULL \
                            OR Robot.miningEndTime <= NOW() \
                            OR Robot.miningEndTime > Robot.rechargeEndTime) \
                     THEN 1 ELSE 0 END, \
                TIMESTAMPDIFF(SECOND, NOW(), Robot.rechargeEndTime), \
                TIMESTAMPDIFF(SECOND, NOW(), Robot.miningEndTime), \
                TIMESTAMPDIFF(SECOND, NOW(), \
                    GREATEST( \
                        COALESCE(Robot.rechargeEndTime, MiningQueue.creationTime), \
                        MiningQueue.creationTime)) \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         WHERE Robot.userId = ? \
           AND (MiningQueue.miningEndTime IS NULL OR MiningQueue.miningEndTime > NOW()) \
         ORDER BY MiningQueue.robotId, MiningQueue.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut states = Vec::with_capacity(rows.len());
    let mut current_robot_id = None;
    let mut robot_time_left = 0_i64;

    for (
        mining_queue_id,
        robot_id,
        recharge_time,
        mining_time,
        is_recharging,
        recharge_seconds_left,
        mining_seconds_left,
        start_seconds_left,
    ) in rows
    {
        let first_queue_for_robot = current_robot_id != Some(robot_id);

        let (status, time_left_seconds, updated_robot_time_left) = derive_mining_queue_state(
            first_queue_for_robot,
            robot_time_left,
            &MiningQueueTimingInput {
                recharge_time,
                mining_time,
                is_recharging: is_recharging != 0,
                recharge_seconds_left,
                mining_seconds_left,
                start_seconds_left,
            },
        );
        robot_time_left = updated_robot_time_left;
        if first_queue_for_robot {
            current_robot_id = Some(robot_id);
        }

        states.push(MiningQueueStateRecord {
            mining_queue_id,
            robot_id,
            status,
            time_left_seconds,
        });
    }

    Ok(states)
}

pub(crate) struct MiningQueueTimingInput {
    pub recharge_time: i32,
    pub mining_time: i32,
    pub is_recharging: bool,
    pub recharge_seconds_left: Option<i64>,
    pub mining_seconds_left: Option<i64>,
    pub start_seconds_left: i64,
}

pub(crate) fn derive_mining_queue_state(
    first_queue_for_robot: bool,
    robot_time_left: i64,
    timing: &MiningQueueTimingInput,
) -> (MiningQueueStatus, i64, i64) {
    if first_queue_for_robot {
        if timing.is_recharging {
            let time_left = timing.recharge_seconds_left.unwrap_or(0).max(1);
            let updated_robot_time_left = time_left + i64::from(timing.mining_time);
            (
                MiningQueueStatus::Recharging,
                time_left,
                updated_robot_time_left,
            )
        } else if let Some(time_left) = timing
            .mining_seconds_left
            .filter(|time_left| *time_left > 0)
        {
            (MiningQueueStatus::Mining, time_left, time_left)
        } else {
            let time_left = timing.start_seconds_left + i64::from(timing.mining_time);

            if time_left > 0 {
                (MiningQueueStatus::Mining, time_left, time_left)
            } else {
                (MiningQueueStatus::Updating, 1, 1)
            }
        }
    } else {
        let updated_robot_time_left =
            robot_time_left + i64::from(timing.recharge_time) + i64::from(timing.mining_time);
        (
            MiningQueueStatus::Queued,
            updated_robot_time_left,
            updated_robot_time_left,
        )
    }
}

pub(crate) fn mining_queue_item_cancelable(
    rally_result_id: Option<i64>,
    mining_end_time_is_null: bool,
    earlier_unfinished_queue_count: i64,
) -> bool {
    rally_result_id.is_none() && mining_end_time_is_null && earlier_unfinished_queue_count > 0
}

#[cfg(test)]
mod tests {
    use super::{MiningQueueTimingInput, derive_mining_queue_state, mining_queue_item_cancelable};
    use crate::MiningQueueStatus;

    #[test]
    fn derive_mining_queue_state_marks_follow_up_items_as_queued() {
        let timing = MiningQueueTimingInput {
            recharge_time: 2,
            mining_time: 5,
            is_recharging: false,
            recharge_seconds_left: None,
            mining_seconds_left: None,
            start_seconds_left: 0,
        };

        let (status, time_left, robot_time_left) = derive_mining_queue_state(false, 10, &timing);

        assert_eq!(status, MiningQueueStatus::Queued);
        assert_eq!(time_left, 17);
        assert_eq!(robot_time_left, 17);
    }

    #[test]
    fn derive_mining_queue_state_marks_active_recharge_as_first_item() {
        let timing = MiningQueueTimingInput {
            recharge_time: 2,
            mining_time: 5,
            is_recharging: true,
            recharge_seconds_left: Some(4),
            mining_seconds_left: None,
            start_seconds_left: 0,
        };

        let (status, time_left, robot_time_left) = derive_mining_queue_state(true, 0, &timing);

        assert_eq!(status, MiningQueueStatus::Recharging);
        assert_eq!(time_left, 4);
        assert_eq!(robot_time_left, 9);
    }

    #[test]
    fn mining_queue_item_cancelable_requires_queued_position() {
        assert!(mining_queue_item_cancelable(None, true, 1));
        assert!(!mining_queue_item_cancelable(Some(9), true, 1));
        assert!(!mining_queue_item_cancelable(None, false, 1));
        assert!(!mining_queue_item_cancelable(None, true, 0));
    }
}
