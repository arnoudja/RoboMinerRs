use crate::output::{escape_state_field, optional_id};
use anyhow::{Context, Result};

pub(crate) async fn activity_states(
    pool: &robominer_db::MySqlPool,
    max_users: i64,
    max_rallies: i64,
) -> Result<()> {
    let recent_users = robominer_db::list_activity_recent_users(pool, max_users)
        .await
        .context("failed to load activity recent users")?;
    let recent_rallies = robominer_db::list_activity_recent_rallies(pool, max_rallies)
        .await
        .context("failed to load activity recent rallies")?;
    let participants = robominer_db::list_activity_recent_rally_participants(pool, max_rallies)
        .await
        .context("failed to load activity rally participants")?;

    for user in recent_users {
        println!(
            "U\t{}\t{}\t{}",
            user.user_id,
            escape_state_field(&user.username),
            user.last_login_time_millis
        );
    }

    for rally in recent_rallies {
        println!(
            "R\t{}\t{}\t{}\t{}\t{}",
            rally.mining_queue_id,
            optional_id(rally.rally_result_id),
            escape_state_field(&rally.mining_area_name),
            escape_state_field(&rally.robot_name),
            escape_state_field(&rally.username)
        );
    }

    for participant in participants {
        println!(
            "P\t{}\t{}\t{}\t{}",
            participant.mining_queue_id,
            participant.player_number,
            escape_state_field(&participant.robot_name),
            escape_state_field(&participant.username)
        );
    }

    Ok(())
}

pub(crate) async fn rally_view_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_user_result: bool,
) -> Result<()> {
    let Some(state) =
        robominer_db::rally_view_state(pool, user_id, rally_result_id, require_user_result)
            .await
            .context("failed to load rally view state")?
    else {
        anyhow::bail!("unknown or inaccessible rally result");
    };
    let ores = robominer_db::list_ores(pool)
        .await
        .context("failed to load rally view ores")?;
    let participants = robominer_db::list_rally_view_participants(pool, rally_result_id)
        .await
        .context("failed to load rally view participants")?;
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

    println!("D\t{}", escape_state_field(&state.result_data));

    for ore in ores {
        println!("O\t{}\t{}", ore.id, escape_state_field(&ore.ore_name));
    }

    for (index, (robot_name, username)) in slots.into_iter().enumerate() {
        println!(
            "S\t{}\t{}\t{}",
            index,
            escape_state_field(&robot_name),
            escape_state_field(&username)
        );
    }

    Ok(())
}
