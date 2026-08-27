use std::path::PathBuf;

use crate::html::assert_html_contains;
use crate::{ServerConfig, mutation_i64};

use super::super::achievements_page;
use super::fixtures::{authenticated_request, form_request};

#[tokio::test(flavor = "current_thread")]
async fn achievements_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = achievements_page(&authenticated_request("/achievements"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[test]
fn form_fields_are_available_to_mutation_parser() {
    let request = form_request("/achievements", "achievementId=42&name=Robo+Miner");

    assert_eq!(mutation_i64(&request, "achievementId"), Some(42));
    assert_eq!(request.form.get("name"), Some(&"Robo Miner".to_string()));

    let mut get_request = request;
    get_request.method = "GET".to_string();
    assert_eq!(mutation_i64(&get_request, "achievementId"), None);
}

#[test]
fn achievement_rejection_messages_match_engine_output() {
    assert_eq!(
        robominer_domain::rejection_messages::claim_achievement_step_rejection_message(
            robominer_db::ClaimAchievementStepRejection::RequirementsNotMet
        ),
        "achievement requirements are not met"
    );
    assert_eq!(
        robominer_domain::rejection_messages::claim_achievement_step_rejection_message(
            robominer_db::ClaimAchievementStepRejection::InvalidDefaultRobotConfiguration
        ),
        "invalid default robot configuration"
    );
}
