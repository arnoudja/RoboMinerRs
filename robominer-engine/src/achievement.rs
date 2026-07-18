use crate::output::{escape_state_field, optional_id};
use anyhow::{Context, Result, anyhow};

pub(crate) async fn claim_achievement_step(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::ClaimAchievementStepRequest,
) -> Result<()> {
    match robominer_db::claim_achievement_step(pool, request)
        .await
        .context("failed to claim achievement step")?
    {
        Ok(result) => {
            println!(
                "Claimed achievement {} step {}",
                result.achievement_id, result.step
            );
            Ok(())
        }
        Err(rejection) => Err(anyhow!(
            "unable to claim achievement step: {}",
            robominer_domain::claim_achievement_step_rejection_message(rejection)
        )),
    }
}

pub(crate) async fn achievement_states(pool: &robominer_db::MySqlPool, user_id: i64) -> Result<()> {
    let claim_states = robominer_db::list_achievement_claim_states_for_user(pool, user_id)
        .await
        .context("failed to load achievement claim states")?;
    let mined_totals = robominer_db::list_user_ore_mined_totals(pool, user_id)
        .await
        .context("failed to load user ore mined totals")?;
    let scores = robominer_db::list_user_best_mining_area_scores(pool, user_id)
        .await
        .context("failed to load user mining area scores")?;

    for claim_state in claim_states {
        println!(
            "C\t{}\t{}",
            claim_state.achievement_id, claim_state.claimable
        );
    }

    for mined_total in mined_totals {
        println!("T\t{}\t{}", mined_total.ore_id, mined_total.amount);
    }

    for score in scores {
        println!("S\t{}\t{}", score.mining_area_id, score.score);
    }

    Ok(())
}

pub(crate) async fn achievement_page_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<()> {
    let robot_count = robominer_db::count_user_robots(pool, user_id)
        .await
        .context("failed to count user robots")?;
    let achievements = robominer_db::list_achievement_page_states_for_user(pool, user_id)
        .await
        .context("failed to load achievement page states")?;
    let total_requirements =
        robominer_db::list_achievement_page_total_requirements_for_user(pool, user_id)
            .await
            .context("failed to load achievement page total requirements")?;
    let score_requirements =
        robominer_db::list_achievement_page_score_requirements_for_user(pool, user_id)
            .await
            .context("failed to load achievement page score requirements")?;

    println!("V\t{robot_count}");

    for achievement in achievements {
        println!(
            "A\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            achievement.achievement_id,
            escape_state_field(&achievement.title),
            escape_state_field(&achievement.description),
            achievement.steps_claimed,
            achievement.number_of_steps,
            achievement.achievement_points_earned,
            achievement.total_achievement_points,
            achievement.step,
            achievement.next_achievement_points,
            achievement.mining_queue_reward,
            achievement.robot_reward,
            optional_id(achievement.ore_id),
            escape_state_field(achievement.ore_name.as_deref().unwrap_or_default()),
            achievement.current_ore_maximum,
            achievement.max_ore_reward,
            achievement.current_depot_maximum,
            achievement.max_depot_reward,
            optional_id(achievement.mining_area_id),
            escape_state_field(achievement.mining_area_name.as_deref().unwrap_or_default()),
            achievement.claimable
        );
    }

    for requirement in total_requirements {
        println!(
            "T\t{}\t{}\t{}\t{}\t{}",
            requirement.achievement_id,
            requirement.ore_id,
            escape_state_field(&requirement.ore_name),
            requirement.amount,
            requirement.current_amount
        );
    }

    for requirement in score_requirements {
        println!(
            "S\t{}\t{}\t{}\t{}\t{}",
            requirement.achievement_id,
            requirement.mining_area_id,
            escape_state_field(&requirement.area_name),
            requirement.minimum_score,
            requirement.current_score
        );
    }

    Ok(())
}
