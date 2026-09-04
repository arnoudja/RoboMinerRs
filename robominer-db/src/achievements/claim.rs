use sqlx::MySqlPool;

use crate::robots::{add_default_robot_for_user, user_robot_count};
use crate::users::touch_user_last_login_time;
use crate::{
    ClaimAchievementStepRejection, ClaimAchievementStepRequest, ClaimedAchievementStep, DbOutcome,
    db_ok, db_reject,
};

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
struct AchievementStepState {
    #[sqlx(rename = "achievementPoints")]
    achievement_points: i32,
    #[sqlx(rename = "miningQueueReward")]
    mining_queue_reward: i32,
    #[sqlx(rename = "robotReward")]
    robot_reward: i32,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: Option<i64>,
    #[sqlx(rename = "oreId")]
    ore_id: Option<i64>,
    #[sqlx(rename = "maxOreReward")]
    max_ore_reward: i32,
    #[sqlx(rename = "maxDepotReward")]
    max_depot_reward: i32,
}

pub async fn claim_achievement_step(
    pool: &MySqlPool,
    request: ClaimAchievementStepRequest,
) -> Result<DbOutcome<ClaimedAchievementStep, ClaimAchievementStepRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let result = claim_achievement_step_in_transaction(&mut transaction, request).await?;

    if result.is_ok() {
        touch_user_last_login_time(&mut transaction, request.user_id).await?;
        transaction.commit().await?;
    } else {
        transaction.rollback().await?;
    }

    Ok(result)
}

pub(crate) async fn claim_achievement_step_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: ClaimAchievementStepRequest,
) -> Result<DbOutcome<ClaimedAchievementStep, ClaimAchievementStepRejection>, sqlx::Error> {
    let Some(steps_claimed) = sqlx::query_scalar::<_, i32>(
        "SELECT stepsClaimed \
         FROM UserAchievement \
         WHERE userId = ? AND achievementId = ? \
         FOR UPDATE",
    )
    .bind(request.user_id)
    .bind(request.achievement_id)
    .fetch_optional(&mut **transaction)
    .await?
    else {
        return db_reject(ClaimAchievementStepRejection::UnknownUserAchievement);
    };

    let next_step = steps_claimed + 1;
    let Some(step) = load_achievement_step(transaction, request.achievement_id, next_step).await?
    else {
        return db_reject(ClaimAchievementStepRejection::NoNextStep);
    };

    if !achievement_requirements_met(
        transaction,
        request.user_id,
        request.achievement_id,
        next_step,
    )
    .await?
    {
        return db_reject(ClaimAchievementStepRejection::RequirementsNotMet);
    }

    sqlx::query(
        "UPDATE UserAchievement \
         SET stepsClaimed = ? \
         WHERE userId = ? AND achievementId = ?",
    )
    .bind(next_step)
    .bind(request.user_id)
    .bind(request.achievement_id)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "UPDATE User \
         SET achievementPoints = achievementPoints + ?, \
             miningQueueSize = miningQueueSize + ? \
         WHERE id = ?",
    )
    .bind(step.achievement_points)
    .bind(step.mining_queue_reward)
    .bind(request.user_id)
    .execute(&mut **transaction)
    .await?;

    if let Some(mining_area_id) = step.mining_area_id {
        sqlx::query(
            "INSERT IGNORE INTO UserMiningArea (userId, miningAreaId) \
             VALUES (?, ?)",
        )
        .bind(request.user_id)
        .bind(mining_area_id)
        .execute(&mut **transaction)
        .await?;
    }

    if let Some(ore_id) = step.ore_id {
        increase_user_ore_maximum(transaction, request.user_id, ore_id, step.max_ore_reward)
            .await?;
        increase_user_depot_maximum(transaction, request.user_id, ore_id, step.max_depot_reward)
            .await?;
    }

    let robot_count = user_robot_count(transaction, request.user_id).await?;
    if i64::from(step.robot_reward) > robot_count {
        add_default_robot_for_user(transaction, request.user_id).await?;
    }

    unlock_successor_achievements(
        transaction,
        request.user_id,
        request.achievement_id,
        next_step,
    )
    .await?;

    db_ok(ClaimedAchievementStep {
        achievement_id: request.achievement_id,
        step: next_step,
    })
}

async fn load_achievement_step(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    achievement_id: i64,
    step: i32,
) -> Result<Option<AchievementStepState>, sqlx::Error> {
    sqlx::query_as::<_, AchievementStepState>(
        "SELECT achievementPoints, miningQueueReward, robotReward, miningAreaId, oreId, \
                maxOreReward, maxDepotReward \
         FROM AchievementStep \
         WHERE achievementId = ? AND step = ?",
    )
    .bind(achievement_id)
    .bind(step)
    .fetch_optional(&mut **transaction)
    .await
}

async fn achievement_requirements_met(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    achievement_id: i64,
    step: i32,
) -> Result<bool, sqlx::Error> {
    let total_requirements = sqlx::query_as::<_, (i64, i32)>(
        "SELECT oreId, amount \
         FROM AchievementStepMiningTotalRequirement \
         WHERE achievementId = ? AND step = ?",
    )
    .bind(achievement_id)
    .bind(step)
    .fetch_all(&mut **transaction)
    .await?;

    for (ore_id, required_amount) in total_requirements {
        let mined: i32 = sqlx::query_scalar(
            "SELECT CAST(COALESCE(SUM(RobotLifetimeResult.amount), 0) AS SIGNED) \
             FROM RobotLifetimeResult \
             INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
             WHERE Robot.userId = ? AND RobotLifetimeResult.oreId = ?",
        )
        .bind(user_id)
        .bind(ore_id)
        .fetch_one(&mut **transaction)
        .await?;

        if mined < required_amount {
            return Ok(false);
        }
    }

    let score_requirements = sqlx::query_as::<_, (i64, f64)>(
        "SELECT miningAreaId, minimumScore \
         FROM AchievementStepMiningScoreRequirement \
         WHERE achievementId = ? AND step = ?",
    )
    .bind(achievement_id)
    .bind(step)
    .fetch_all(&mut **transaction)
    .await?;

    for (mining_area_id, minimum_score) in score_requirements {
        let score: f64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(RobotMiningAreaScore.score), 0.0) \
             FROM RobotMiningAreaScore \
             INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
             WHERE Robot.userId = ? AND RobotMiningAreaScore.miningAreaId = ?",
        )
        .bind(user_id)
        .bind(mining_area_id)
        .fetch_one(&mut **transaction)
        .await?;

        if !crate::achievement_score_meets_requirement(score, minimum_score) {
            return Ok(false);
        }
    }

    let depot_requirements = sqlx::query_as::<_, (i64, i32)>(
        "SELECT oreId, amount \
         FROM AchievementStepDepotTotalRequirement \
         WHERE achievementId = ? AND step = ?",
    )
    .bind(achievement_id)
    .bind(step)
    .fetch_all(&mut **transaction)
    .await?;

    for (ore_id, required_amount) in depot_requirements {
        let deposited =
            super::read::user_depot_total_for_ore(&mut **transaction, user_id, ore_id).await?;

        if deposited < required_amount {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn increase_user_ore_maximum(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    ore_id: i64,
    max_ore_reward: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
         VALUES (?, ?, 0, ?) \
         ON DUPLICATE KEY UPDATE maxAllowed = GREATEST(maxAllowed, VALUES(maxAllowed))",
    )
    .bind(user_id)
    .bind(ore_id)
    .bind(max_ore_reward)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn increase_user_depot_maximum(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    ore_id: i64,
    max_depot_reward: i32,
) -> Result<(), sqlx::Error> {
    use crate::INITIAL_ORE_WALLET_MAX;

    sqlx::query(
        "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed, depotMaxAllowed) \
         VALUES (?, ?, 0, ?, ?) \
         ON DUPLICATE KEY UPDATE \
         depotMaxAllowed = GREATEST(depotMaxAllowed, VALUES(depotMaxAllowed))",
    )
    .bind(user_id)
    .bind(ore_id)
    .bind(INITIAL_ORE_WALLET_MAX)
    .bind(max_depot_reward)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn unlock_successor_achievements(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    achievement_id: i64,
    claimed_step: i32,
) -> Result<(), sqlx::Error> {
    let successors = sqlx::query_as::<_, (i64,)>(
        "SELECT successorId \
         FROM AchievementPredecessor \
         WHERE predecessorId = ? AND predecessorStep = ?",
    )
    .bind(achievement_id)
    .bind(claimed_step)
    .fetch_all(&mut **transaction)
    .await?;

    for (successor_id,) in successors {
        super::unlock::grant_successor_if_eligible(transaction, user_id, successor_id).await?;
    }

    Ok(())
}
