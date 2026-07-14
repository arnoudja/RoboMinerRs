use sqlx::MySqlPool;

use crate::users::touch_user_last_login_time;
use sqlx::Row;

use crate::robots::{add_default_robot_for_user, user_robot_count};
use crate::{
    AchievementClaimStateRecord, AchievementPagePointsSummaryRecord,
    AchievementPageScoreRequirementRecord, AchievementPageStateRecord,
    AchievementPageTotalRequirementRecord, ClaimAchievementStepRejection,
    ClaimAchievementStepRequest, ClaimedAchievementStep, INITIAL_ORE_WALLET_MAX,
    UserMiningAreaScoreRecord, UserOreMinedRecord,
};

#[derive(Debug, Clone, Copy)]
struct AchievementStepState {
    achievement_points: i32,
    mining_queue_reward: i32,
    robot_reward: i32,
    mining_area_id: Option<i64>,
    ore_id: Option<i64>,
    max_ore_reward: i32,
}

pub async fn list_achievement_claim_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementClaimStateRecord>, sqlx::Error> {
    reconcile_successor_unlocks(pool, user_id).await?;

    sqlx::query_as::<_, (i64, i8)>(
        "SELECT UserAchievement.achievementId, \
                CASE WHEN AchievementStep.achievementId IS NOT NULL \
                       AND NOT EXISTS \
                         (SELECT 1 \
                          FROM AchievementStepMiningTotalRequirement \
                          WHERE AchievementStepMiningTotalRequirement.achievementId = AchievementStep.achievementId \
                            AND AchievementStepMiningTotalRequirement.step = AchievementStep.step \
                            AND AchievementStepMiningTotalRequirement.amount > \
                              (SELECT CAST(COALESCE(SUM(RobotLifetimeResult.amount), 0) AS SIGNED) \
                               FROM RobotLifetimeResult \
                               INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotLifetimeResult.oreId = AchievementStepMiningTotalRequirement.oreId)) \
                       AND NOT EXISTS \
                         (SELECT 1 \
                          FROM AchievementStepMiningScoreRequirement \
                          WHERE AchievementStepMiningScoreRequirement.achievementId = AchievementStep.achievementId \
                            AND AchievementStepMiningScoreRequirement.step = AchievementStep.step \
                            AND AchievementStepMiningScoreRequirement.minimumScore > \
                              (SELECT COALESCE(MAX(RobotMiningAreaScore.score), 0.0) \
                               FROM RobotMiningAreaScore \
                               INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotMiningAreaScore.miningAreaId = AchievementStepMiningScoreRequirement.miningAreaId)) \
                     THEN 1 ELSE 0 END \
         FROM UserAchievement \
         LEFT JOIN AchievementStep \
           ON AchievementStep.achievementId = UserAchievement.achievementId \
          AND AchievementStep.step = UserAchievement.stepsClaimed + 1 \
         WHERE UserAchievement.userId = ? \
         ORDER BY UserAchievement.achievementId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(achievement_id, claimable)| AchievementClaimStateRecord {
                achievement_id,
                claimable: claimable != 0,
            })
            .collect()
    })
}

pub async fn list_user_ore_mined_totals(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserOreMinedRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32)>(
        "SELECT RobotLifetimeResult.oreId, \
                CAST(COALESCE(SUM(RobotLifetimeResult.amount), 0) AS SIGNED) \
         FROM RobotLifetimeResult \
         INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
         WHERE Robot.userId = ? \
         GROUP BY RobotLifetimeResult.oreId \
         ORDER BY RobotLifetimeResult.oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(ore_id, amount)| UserOreMinedRecord { ore_id, amount })
            .collect()
    })
}

pub async fn list_user_best_mining_area_scores(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserMiningAreaScoreRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, f64)>(
        "SELECT RobotMiningAreaScore.miningAreaId, \
                COALESCE(MAX(RobotMiningAreaScore.score), 0.0) \
         FROM RobotMiningAreaScore \
         INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
         WHERE Robot.userId = ? \
         GROUP BY RobotMiningAreaScore.miningAreaId \
         ORDER BY RobotMiningAreaScore.miningAreaId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(mining_area_id, score)| UserMiningAreaScoreRecord {
                mining_area_id,
                score,
            })
            .collect()
    })
}
pub async fn list_achievement_page_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementPageStateRecord>, sqlx::Error> {
    reconcile_successor_unlocks(pool, user_id).await?;

    let query = format!(
        "SELECT Achievement.id AS achievementId, \
                Achievement.title AS title, \
                Achievement.description AS description, \
                UserAchievement.stepsClaimed AS stepsClaimed, \
                (SELECT COUNT(*) FROM AchievementStep AllSteps \
                 WHERE AllSteps.achievementId = Achievement.id) AS numberOfSteps, \
                CAST(COALESCE((SELECT SUM(ClaimedStep.achievementPoints) \
                               FROM AchievementStep ClaimedStep \
                               WHERE ClaimedStep.achievementId = Achievement.id \
                                 AND ClaimedStep.step <= UserAchievement.stepsClaimed), 0) AS SIGNED) \
                  AS achievementPointsEarned, \
                CAST(COALESCE((SELECT SUM(AllPoints.achievementPoints) \
                               FROM AchievementStep AllPoints \
                               WHERE AllPoints.achievementId = Achievement.id), 0) AS SIGNED) \
                  AS totalAchievementPoints, \
                AchievementStep.step AS step, \
                AchievementStep.achievementPoints AS nextAchievementPoints, \
                AchievementStep.miningQueueReward AS miningQueueReward, \
                AchievementStep.robotReward AS robotReward, \
                Ore.id AS oreId, \
                Ore.oreName AS oreName, \
                CAST(COALESCE((SELECT UserOreAsset.maxAllowed \
                               FROM UserOreAsset \
                               WHERE UserOreAsset.userId = UserAchievement.userId \
                                 AND UserOreAsset.oreId = AchievementStep.oreId), {initial}) AS SIGNED) \
                  AS currentOreMaximum, \
                AchievementStep.maxOreReward AS maxOreReward, \
                MiningArea.id AS miningAreaId, \
                MiningArea.areaName AS miningAreaName, \
                CASE WHEN NOT EXISTS \
                         (SELECT 1 \
                          FROM AchievementStepMiningTotalRequirement \
                          WHERE AchievementStepMiningTotalRequirement.achievementId = AchievementStep.achievementId \
                            AND AchievementStepMiningTotalRequirement.step = AchievementStep.step \
                            AND AchievementStepMiningTotalRequirement.amount > \
                              (SELECT CAST(COALESCE(SUM(RobotLifetimeResult.amount), 0) AS SIGNED) \
                               FROM RobotLifetimeResult \
                               INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotLifetimeResult.oreId = AchievementStepMiningTotalRequirement.oreId)) \
                       AND NOT EXISTS \
                         (SELECT 1 \
                          FROM AchievementStepMiningScoreRequirement \
                          WHERE AchievementStepMiningScoreRequirement.achievementId = AchievementStep.achievementId \
                            AND AchievementStepMiningScoreRequirement.step = AchievementStep.step \
                            AND AchievementStepMiningScoreRequirement.minimumScore > \
                              (SELECT COALESCE(MAX(RobotMiningAreaScore.score), 0.0) \
                               FROM RobotMiningAreaScore \
                               INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotMiningAreaScore.miningAreaId = AchievementStepMiningScoreRequirement.miningAreaId)) \
                     THEN 1 ELSE 0 END AS claimable \
         FROM UserAchievement \
         INNER JOIN Achievement ON Achievement.id = UserAchievement.achievementId \
         INNER JOIN AchievementStep \
           ON AchievementStep.achievementId = Achievement.id \
          AND AchievementStep.step = UserAchievement.stepsClaimed + 1 \
         LEFT OUTER JOIN Ore ON Ore.id = AchievementStep.oreId \
         LEFT OUTER JOIN MiningArea ON MiningArea.id = AchievementStep.miningAreaId \
         WHERE UserAchievement.userId = ? \
         ORDER BY UserAchievement.achievementId",
        initial = INITIAL_ORE_WALLET_MAX,
    );

    sqlx::query(&query)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|row| {
                Ok(AchievementPageStateRecord {
                    achievement_id: row.try_get("achievementId")?,
                    title: row.try_get("title")?,
                    description: row.try_get("description")?,
                    steps_claimed: row.try_get("stepsClaimed")?,
                    number_of_steps: row.try_get("numberOfSteps")?,
                    achievement_points_earned: row.try_get("achievementPointsEarned")?,
                    total_achievement_points: row.try_get("totalAchievementPoints")?,
                    step: row.try_get("step")?,
                    next_achievement_points: row.try_get("nextAchievementPoints")?,
                    mining_queue_reward: row.try_get("miningQueueReward")?,
                    robot_reward: row.try_get("robotReward")?,
                    ore_id: row.try_get("oreId")?,
                    ore_name: row.try_get("oreName")?,
                    current_ore_maximum: row.try_get("currentOreMaximum")?,
                    max_ore_reward: row.try_get("maxOreReward")?,
                    mining_area_id: row.try_get("miningAreaId")?,
                    mining_area_name: row.try_get("miningAreaName")?,
                    claimable: row.try_get::<i8, _>("claimable")? != 0,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
    })?
}

pub async fn load_achievement_page_points_summary_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<AchievementPagePointsSummaryRecord, sqlx::Error> {
    let points_earned: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(AchievementStep.achievementPoints), 0) AS SIGNED) \
         FROM UserAchievement \
         INNER JOIN AchievementStep \
           ON AchievementStep.achievementId = UserAchievement.achievementId \
          AND AchievementStep.step <= UserAchievement.stepsClaimed \
         WHERE UserAchievement.userId = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    let points_achievable: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(achievementPoints), 0) AS SIGNED) FROM AchievementStep",
    )
    .fetch_one(pool)
    .await?;

    Ok(AchievementPagePointsSummaryRecord {
        points_earned,
        points_achievable,
    })
}

pub async fn list_achievement_page_total_requirements_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementPageTotalRequirementRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, i32, i32)>(
        "SELECT UserAchievement.achievementId, Ore.id, Ore.oreName, \
                AchievementStepMiningTotalRequirement.amount, \
                CAST(COALESCE((SELECT SUM(RobotLifetimeResult.amount) \
                               FROM RobotLifetimeResult \
                               INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotLifetimeResult.oreId = AchievementStepMiningTotalRequirement.oreId), 0) AS SIGNED) \
         FROM UserAchievement \
         INNER JOIN AchievementStep \
           ON AchievementStep.achievementId = UserAchievement.achievementId \
          AND AchievementStep.step = UserAchievement.stepsClaimed + 1 \
         INNER JOIN AchievementStepMiningTotalRequirement \
           ON AchievementStepMiningTotalRequirement.achievementId = AchievementStep.achievementId \
          AND AchievementStepMiningTotalRequirement.step = AchievementStep.step \
         INNER JOIN Ore ON Ore.id = AchievementStepMiningTotalRequirement.oreId \
         WHERE UserAchievement.userId = ? \
         ORDER BY UserAchievement.achievementId, Ore.id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(achievement_id, ore_id, ore_name, amount, current_amount)| {
                    AchievementPageTotalRequirementRecord {
                        achievement_id,
                        ore_id,
                        ore_name,
                        amount,
                        current_amount,
                    }
                },
            )
            .collect()
    })
}

pub async fn list_achievement_page_score_requirements_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementPageScoreRequirementRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, f64, f64)>(
        "SELECT UserAchievement.achievementId, MiningArea.id, MiningArea.areaName, \
                AchievementStepMiningScoreRequirement.minimumScore, \
                COALESCE((SELECT MAX(RobotMiningAreaScore.score) \
                          FROM RobotMiningAreaScore \
                          INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
                          WHERE Robot.userId = UserAchievement.userId \
                            AND RobotMiningAreaScore.miningAreaId = AchievementStepMiningScoreRequirement.miningAreaId), 0.0) \
         FROM UserAchievement \
         INNER JOIN AchievementStep \
           ON AchievementStep.achievementId = UserAchievement.achievementId \
          AND AchievementStep.step = UserAchievement.stepsClaimed + 1 \
         INNER JOIN AchievementStepMiningScoreRequirement \
           ON AchievementStepMiningScoreRequirement.achievementId = AchievementStep.achievementId \
          AND AchievementStepMiningScoreRequirement.step = AchievementStep.step \
         INNER JOIN MiningArea ON MiningArea.id = AchievementStepMiningScoreRequirement.miningAreaId \
         WHERE UserAchievement.userId = ? \
         ORDER BY UserAchievement.achievementId, MiningArea.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(achievement_id, mining_area_id, area_name, minimum_score, current_score)| {
                    AchievementPageScoreRequirementRecord {
                        achievement_id,
                        mining_area_id,
                        area_name,
                        minimum_score,
                        current_score,
                    }
                },
            )
            .collect()
    })
}
pub async fn claim_achievement_step(
    pool: &MySqlPool,
    request: ClaimAchievementStepRequest,
) -> Result<Result<ClaimedAchievementStep, ClaimAchievementStepRejection>, sqlx::Error> {
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
) -> Result<Result<ClaimedAchievementStep, ClaimAchievementStepRejection>, sqlx::Error> {
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
        return Ok(Err(ClaimAchievementStepRejection::UnknownUserAchievement));
    };

    let next_step = steps_claimed + 1;
    let Some(step) = load_achievement_step(transaction, request.achievement_id, next_step).await?
    else {
        return Ok(Err(ClaimAchievementStepRejection::NoNextStep));
    };

    if !achievement_requirements_met(
        transaction,
        request.user_id,
        request.achievement_id,
        next_step,
    )
    .await?
    {
        return Ok(Err(ClaimAchievementStepRejection::RequirementsNotMet));
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

    Ok(Ok(ClaimedAchievementStep {
        achievement_id: request.achievement_id,
        step: next_step,
    }))
}
async fn load_achievement_step(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    achievement_id: i64,
    step: i32,
) -> Result<Option<AchievementStepState>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT achievementPoints, miningQueueReward, robotReward, miningAreaId, oreId, \
                maxOreReward \
         FROM AchievementStep \
         WHERE achievementId = ? AND step = ?",
    )
    .bind(achievement_id)
    .bind(step)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(|row| {
        Ok(AchievementStepState {
            achievement_points: row.try_get("achievementPoints")?,
            mining_queue_reward: row.try_get("miningQueueReward")?,
            robot_reward: row.try_get("robotReward")?,
            mining_area_id: row.try_get("miningAreaId")?,
            ore_id: row.try_get("oreId")?,
            max_ore_reward: row.try_get("maxOreReward")?,
        })
    })
    .transpose()
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

        if score < minimum_score {
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
        grant_successor_if_eligible(transaction, user_id, successor_id).await?;
    }

    Ok(())
}

pub async fn reconcile_successor_unlocks(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    reconcile_successor_unlocks_in_transaction(&mut transaction, user_id).await?;
    transaction.commit().await?;
    Ok(())
}

async fn reconcile_successor_unlocks_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let successor_ids = sqlx::query_scalar::<_, i64>(
        "SELECT DISTINCT successorId \
         FROM AchievementPredecessor \
         WHERE NOT EXISTS \
           (SELECT 1 \
            FROM UserAchievement \
            WHERE UserAchievement.userId = ? \
              AND UserAchievement.achievementId = AchievementPredecessor.successorId)",
    )
    .bind(user_id)
    .fetch_all(&mut **transaction)
    .await?;

    for successor_id in successor_ids {
        grant_successor_if_eligible(transaction, user_id, successor_id).await?;
    }

    Ok(())
}

async fn grant_successor_if_eligible(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    successor_id: i64,
) -> Result<(), sqlx::Error> {
    if successor_requirements_met(transaction, user_id, successor_id).await? {
        sqlx::query(
            "INSERT IGNORE INTO UserAchievement (userId, achievementId, stepsClaimed) \
             VALUES (?, ?, 0)",
        )
        .bind(user_id)
        .bind(successor_id)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

async fn successor_requirements_met(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    achievement_id: i64,
) -> Result<bool, sqlx::Error> {
    let predecessors = sqlx::query_as::<_, (i64, i32)>(
        "SELECT predecessorId, predecessorStep \
         FROM AchievementPredecessor \
         WHERE successorId = ?",
    )
    .bind(achievement_id)
    .fetch_all(&mut **transaction)
    .await?;

    for (predecessor_id, predecessor_step) in predecessors {
        let steps_claimed: Option<i32> = sqlx::query_scalar(
            "SELECT stepsClaimed \
             FROM UserAchievement \
             WHERE userId = ? AND achievementId = ?",
        )
        .bind(user_id)
        .bind(predecessor_id)
        .fetch_optional(&mut **transaction)
        .await?;

        if !predecessor_step_met(steps_claimed.unwrap_or_default(), predecessor_step) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) fn predecessor_step_met(steps_claimed: i32, required_step: i32) -> bool {
    steps_claimed >= required_step
}

#[cfg(test)]
mod tests {
    use super::predecessor_step_met;

    fn successor_predecessors_met(
        predecessors: &[(i64, i32)],
        claimed_steps: &[(i64, i32)],
    ) -> bool {
        predecessors.iter().all(|(predecessor_id, required_step)| {
            let steps_claimed = claimed_steps
                .iter()
                .find(|(id, _)| id == predecessor_id)
                .map(|(_, steps)| *steps)
                .unwrap_or(0);
            predecessor_step_met(steps_claimed, *required_step)
        })
    }

    #[test]
    fn predecessor_step_met_requires_minimum_claimed_steps() {
        assert!(predecessor_step_met(2, 2));
        assert!(predecessor_step_met(3, 2));
        assert!(!predecessor_step_met(1, 2));
        assert!(!predecessor_step_met(0, 1));
    }

    #[test]
    fn successor_predecessors_met_requires_all_predecessors() {
        let predecessors = [(10, 1), (20, 2)];
        assert!(successor_predecessors_met(
            &predecessors,
            &[(10, 1), (20, 2)],
        ));
        assert!(successor_predecessors_met(
            &predecessors,
            &[(10, 2), (20, 3)],
        ));
        assert!(!successor_predecessors_met(
            &predecessors,
            &[(10, 1), (20, 1)],
        ));
        assert!(!successor_predecessors_met(&predecessors, &[(10, 1)]));
    }
}
