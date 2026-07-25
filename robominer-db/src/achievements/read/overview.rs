use sqlx::MySqlPool;
use sqlx::Row;

use crate::AchievementOverviewTrackRecord;

/// Unlocked achievement tracks for a read-only player overview.
///
/// Unlike the claim-page loader, this includes completed tracks and does not
/// reconcile successor unlocks (viewing another player must stay read-only).
pub async fn list_achievement_overview_tracks_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementOverviewTrackRecord>, sqlx::Error> {
    sqlx::query(
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
                  AS pointsEarned, \
                CAST(COALESCE((SELECT SUM(AllPoints.achievementPoints) \
                               FROM AchievementStep AllPoints \
                               WHERE AllPoints.achievementId = Achievement.id), 0) AS SIGNED) \
                  AS totalPoints \
         FROM UserAchievement \
         INNER JOIN Achievement ON Achievement.id = UserAchievement.achievementId \
         WHERE UserAchievement.userId = ? \
         ORDER BY UserAchievement.achievementId DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|row| {
                Ok(AchievementOverviewTrackRecord {
                    achievement_id: row.try_get("achievementId")?,
                    title: row.try_get("title")?,
                    description: row.try_get("description")?,
                    steps_claimed: row.try_get("stepsClaimed")?,
                    number_of_steps: row.try_get("numberOfSteps")?,
                    points_earned: row.try_get("pointsEarned")?,
                    total_points: row.try_get("totalPoints")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
    })?
}
