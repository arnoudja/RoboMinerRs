use sqlx::MySqlPool;
use sqlx::Row;

use crate::{
    AchievementClaimStateRecord, AchievementPagePointsSummaryRecord,
    AchievementPageScoreRequirementRecord, AchievementPageStateRecord,
    AchievementPageTotalRequirementRecord, INITIAL_ORE_WALLET_MAX, UserMiningAreaScoreRecord,
    UserOreMinedRecord,
};

pub async fn list_achievement_claim_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementClaimStateRecord>, sqlx::Error> {
    super::unlock::reconcile_successor_unlocks(pool, user_id).await?;

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
    super::unlock::reconcile_successor_unlocks(pool, user_id).await?;

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
                CAST(COALESCE((SELECT UserOreAsset.depotMaxAllowed \
                               FROM UserOreAsset \
                               WHERE UserOreAsset.userId = UserAchievement.userId \
                                 AND UserOreAsset.oreId = AchievementStep.oreId), 0) AS SIGNED) \
                  AS currentDepotMaximum, \
                AchievementStep.maxDepotReward AS maxDepotReward, \
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
                        current_depot_maximum: row.try_get("currentDepotMaximum")?,
                        max_depot_reward: row.try_get("maxDepotReward")?,
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
