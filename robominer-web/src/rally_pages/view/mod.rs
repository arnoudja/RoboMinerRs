use super::RallyViewPageState;

pub async fn load_rally_view_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_user_result: bool,
) -> Result<Option<RallyViewPageState>, robominer_domain::DomainError> {
    rally_view_state(
        pool,
        user_id,
        rally_result_id,
        require_user_result,
        require_user_result,
    )
    .await
}

pub async fn load_user_rally_view_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    rally_result_id: i64,
) -> Result<Option<RallyViewPageState>, robominer_domain::DomainError> {
    rally_view_state(pool, user_id, rally_result_id, true, true).await
}

async fn rally_view_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_user_result: bool,
    require_claimed_viewer_result: bool,
) -> Result<Option<RallyViewPageState>, robominer_domain::DomainError> {
    let Some(state) =
        robominer_db::rally_view_state(pool, user_id, rally_result_id, require_user_result).await?
    else {
        return Ok(None);
    };
    let ores = robominer_db::list_ores(pool).await?;
    let metadata = robominer_db::rally_view_metadata(
        pool,
        user_id,
        rally_result_id,
        require_claimed_viewer_result,
    )
    .await?;
    let Some(metadata) = metadata else {
        return Ok(None);
    };
    let participants = robominer_db::list_rally_view_participants(pool, rally_result_id).await?;
    let mut slots = [
        (state.ai_robot_name.clone(), state.ai_username.clone()),
        (state.ai_robot_name.clone(), state.ai_username.clone()),
        (state.ai_robot_name.clone(), state.ai_username.clone()),
        (state.ai_robot_name.clone(), state.ai_username.clone()),
    ];

    for participant in participants {
        if let Ok(index) = usize::try_from(participant.player_number)
            && index < slots.len()
        {
            slots[index] = (participant.robot_name, participant.username);
        }
    }

    let viewer_source_code = match metadata.viewer_robot_id {
        Some(robot_id) => robominer_db::get_robot(pool, robot_id)
            .await?
            .map(|robot| robot.source_code),
        None => None,
    };

    Ok(Some(RallyViewPageState {
        result_data: state.result_data,
        ores,
        slots,
        mining_area_name: metadata.mining_area_name,
        viewer_player_number: metadata.viewer_player_number,
        viewer_robot_id: metadata.viewer_robot_id,
        viewer_robot_name: metadata.viewer_robot_name,
        viewer_score: metadata.viewer_score,
        viewer_total_reward: metadata.viewer_total_reward,
        viewer_result_claimed: metadata.viewer_result_claimed,
        viewer_source_code,
    }))
}
pub fn valid_mining_results_return_to(value: &str) -> Option<&str> {
    if value.is_empty() || value.contains("://") || value.starts_with('/') {
        None
    } else {
        Some(value)
    }
}

pub(super) mod render;
