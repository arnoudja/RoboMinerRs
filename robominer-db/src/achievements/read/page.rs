use crate::assert_sql_safe;
use sqlx::MySqlPool;

use crate::{
    AchievementPageDepotTotalRequirementRecord, AchievementPagePointsSummaryRecord,
    AchievementPageScoreRequirementRecord, AchievementPageStateRecord,
    AchievementPageTotalRequirementRecord, INITIAL_ORE_WALLET_MAX,
};

#[derive(sqlx::FromRow)]
struct AchievementPageStateRow {
    #[sqlx(rename = "achievementId")]
    achievement_id: i64,
    title: String,
    description: String,
    #[sqlx(rename = "stepsClaimed")]
    steps_claimed: i32,
    #[sqlx(rename = "numberOfSteps")]
    number_of_steps: i64,
    #[sqlx(rename = "achievementPointsEarned")]
    achievement_points_earned: i64,
    #[sqlx(rename = "totalAchievementPoints")]
    total_achievement_points: i64,
    step: i32,
    #[sqlx(rename = "nextAchievementPoints")]
    next_achievement_points: i32,
    #[sqlx(rename = "miningQueueReward")]
    mining_queue_reward: i32,
    #[sqlx(rename = "robotReward")]
    robot_reward: i32,
    #[sqlx(rename = "oreId")]
    ore_id: Option<i64>,
    #[sqlx(rename = "oreName")]
    ore_name: Option<String>,
    #[sqlx(rename = "currentOreMaximum")]
    current_ore_maximum: i32,
    #[sqlx(rename = "maxOreReward")]
    max_ore_reward: i32,
    #[sqlx(rename = "currentDepotMaximum")]
    current_depot_maximum: i32,
    #[sqlx(rename = "maxDepotReward")]
    max_depot_reward: i32,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: Option<i64>,
    #[sqlx(rename = "miningAreaName")]
    mining_area_name: Option<String>,
    claimable: i8,
}

impl From<AchievementPageStateRow> for AchievementPageStateRecord {
    fn from(row: AchievementPageStateRow) -> Self {
        Self {
            achievement_id: row.achievement_id,
            title: row.title,
            description: row.description,
            steps_claimed: row.steps_claimed,
            number_of_steps: row.number_of_steps,
            achievement_points_earned: row.achievement_points_earned,
            total_achievement_points: row.total_achievement_points,
            step: row.step,
            next_achievement_points: row.next_achievement_points,
            mining_queue_reward: row.mining_queue_reward,
            robot_reward: row.robot_reward,
            ore_id: row.ore_id,
            ore_name: row.ore_name,
            current_ore_maximum: row.current_ore_maximum,
            max_ore_reward: row.max_ore_reward,
            current_depot_maximum: row.current_depot_maximum,
            max_depot_reward: row.max_depot_reward,
            mining_area_id: row.mining_area_id,
            mining_area_name: row.mining_area_name,
            claimable: row.claimable != 0,
        }
    }
}

pub async fn list_achievement_page_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementPageStateRecord>, sqlx::Error> {
    super::super::unlock::reconcile_successor_unlocks(pool, user_id).await?;

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

    sqlx::query_as::<_, AchievementPageStateRow>(assert_sql_safe(query))
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(AchievementPageStateRecord::from)
                .collect()
        })
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
    sqlx::query_as::<_, (i64, i64, String, f64, f64, Option<String>)>(
        "SELECT UserAchievement.achievementId, MiningArea.id, MiningArea.areaName, \
                AchievementStepMiningScoreRequirement.minimumScore, \
                COALESCE((SELECT MAX(RobotMiningAreaScore.score) \
                          FROM RobotMiningAreaScore \
                          INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
                          WHERE Robot.userId = UserAchievement.userId \
                            AND RobotMiningAreaScore.miningAreaId = AchievementStepMiningScoreRequirement.miningAreaId), 0.0), \
                (SELECT Robot.robotName \
                 FROM RobotMiningAreaScore \
                 INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
                 WHERE Robot.userId = UserAchievement.userId \
                   AND RobotMiningAreaScore.miningAreaId = AchievementStepMiningScoreRequirement.miningAreaId \
                 ORDER BY RobotMiningAreaScore.score DESC, Robot.id ASC \
                 LIMIT 1) \
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
                |(
                    achievement_id,
                    mining_area_id,
                    area_name,
                    minimum_score,
                    current_score,
                    current_score_robot_name,
                )| {
                    AchievementPageScoreRequirementRecord {
                        achievement_id,
                        mining_area_id,
                        area_name,
                        minimum_score,
                        current_score,
                        current_score_robot_name,
                    }
                },
            )
            .collect()
    })
}

pub async fn list_achievement_page_depot_total_requirements_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementPageDepotTotalRequirementRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, i32, i32)>(
        "SELECT UserAchievement.achievementId, Ore.id, Ore.oreName, \
                AchievementStepDepotTotalRequirement.amount, \
                CAST(COALESCE((SELECT SUM(RobotLifetimeResult.depotAmount) \
                               FROM RobotLifetimeResult \
                               INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
                               WHERE Robot.userId = UserAchievement.userId \
                                 AND RobotLifetimeResult.oreId = AchievementStepDepotTotalRequirement.oreId), 0) AS SIGNED) \
         FROM UserAchievement \
         INNER JOIN AchievementStep \
           ON AchievementStep.achievementId = UserAchievement.achievementId \
          AND AchievementStep.step = UserAchievement.stepsClaimed + 1 \
         INNER JOIN AchievementStepDepotTotalRequirement \
           ON AchievementStepDepotTotalRequirement.achievementId = AchievementStep.achievementId \
          AND AchievementStepDepotTotalRequirement.step = AchievementStep.step \
         INNER JOIN Ore ON Ore.id = AchievementStepDepotTotalRequirement.oreId \
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
                    AchievementPageDepotTotalRequirementRecord {
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
