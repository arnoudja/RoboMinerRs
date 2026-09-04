use sqlx::MySqlPool;

use crate::AchievementOverviewTrackRecord;

#[derive(sqlx::FromRow)]
struct AchievementOverviewTrackRow {
    #[sqlx(rename = "achievementId")]
    achievement_id: i64,
    title: String,
    description: String,
    #[sqlx(rename = "stepsClaimed")]
    steps_claimed: i32,
    #[sqlx(rename = "numberOfSteps")]
    number_of_steps: i64,
    #[sqlx(rename = "pointsEarned")]
    points_earned: i64,
    #[sqlx(rename = "totalPoints")]
    total_points: i64,
}

impl From<AchievementOverviewTrackRow> for AchievementOverviewTrackRecord {
    fn from(row: AchievementOverviewTrackRow) -> Self {
        Self {
            achievement_id: row.achievement_id,
            title: row.title,
            description: row.description,
            steps_claimed: row.steps_claimed,
            number_of_steps: row.number_of_steps,
            points_earned: row.points_earned,
            total_points: row.total_points,
        }
    }
}

/// Unlocked achievement tracks for a read-only player overview.
///
/// Unlike the claim-page loader, this includes completed tracks and does not
/// reconcile successor unlocks (viewing another player must stay read-only).
pub async fn list_achievement_overview_tracks_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<AchievementOverviewTrackRecord>, sqlx::Error> {
    sqlx::query_as::<_, AchievementOverviewTrackRow>(
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
            .map(AchievementOverviewTrackRecord::from)
            .collect()
    })
}
