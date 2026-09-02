use sqlx::MySqlPool;

use crate::AchievementClaimStateRecord;

pub async fn list_achievement_claim_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementClaimStateRecord>, sqlx::Error> {
    super::super::unlock::reconcile_successor_unlocks(pool, user_id).await?;

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
                            AND ROUND(AchievementStepMiningScoreRequirement.minimumScore, 1) > \
                              ROUND((SELECT COALESCE(MAX(RobotMiningAreaScore.score), 0.0) \
                               FROM RobotMiningAreaScore \
                               INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotMiningAreaScore.miningAreaId = AchievementStepMiningScoreRequirement.miningAreaId), 1)) \
                       AND NOT EXISTS \
                         (SELECT 1 \
                          FROM AchievementStepDepotTotalRequirement \
                          WHERE AchievementStepDepotTotalRequirement.achievementId = AchievementStep.achievementId \
                            AND AchievementStepDepotTotalRequirement.step = AchievementStep.step \
                            AND AchievementStepDepotTotalRequirement.amount > \
                              (SELECT CAST(COALESCE(SUM(RobotLifetimeResult.depotAmount), 0) AS SIGNED) \
                               FROM RobotLifetimeResult \
                               INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotLifetimeResult.oreId = AchievementStepDepotTotalRequirement.oreId)) \
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
