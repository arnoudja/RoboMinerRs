mod support;

use robominer_test_support::{CancelMiningQueueFixture, insert_row_id};
use robominer_web::test_support::{format_authenticated_cookie, route};
use serial_test::serial;
use support::{
    WebSmokeFixture, cookie_header, ensure_session_configured, get_request, get_request_query,
    response_body, server_config,
};

use std::collections::HashMap;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn read_model_pages_render_with_seeded_data() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping read-model web tests: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let database_url = std::env::var("ROBOMINER_DATABASE_URL").expect("database url");
    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = WebSmokeFixture::create(&pool).await;
    let config = server_config(pool.clone());
    let cookie = cookie_header(&fixture.login(&config).await);

    for (path, marker) in [
        ("/miningResults", "mining-results-page"),
        ("/leaderboard", "leaderboard-page"),
        ("/miningAreaOverview", "mining-area-atlas-title"),
        ("/activity", "activity-page"),
    ] {
        let response = route(&support::get_request(path, Some(&cookie)), &config).await;
        let body = response_body(&response);

        assert_eq!(response.status, 200, "{path} should render");
        assert!(
            body.contains(marker),
            "expected {path} to contain {marker}:\n{body}"
        );
    }

    fixture.cleanup(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn rally_view_renders_from_mining_results_and_activity() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping rally-view web tests: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let database_url = std::env::var("ROBOMINER_DATABASE_URL").expect("database url");
    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = CancelMiningQueueFixture::create(&pool).await;
    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('var myRobots = {robot: []};')"),
    )
    .await;
    fixture.rally_result_id.set(Some(rally_result_id));

    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, playerNumber, miningEndTime, claimed) \
             VALUES (?, ?, ?, 0, TIMESTAMPADD(SECOND, -10, NOW()), TRUE)",
        )
        .bind(fixture.mining_area_id)
        .bind(fixture.robot_id)
        .bind(rally_result_id),
    )
    .await;
    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, playerNumber, miningEndTime) \
             VALUES (?, ?, ?, 1, TIMESTAMPADD(SECOND, -10, NOW()))",
        )
        .bind(fixture.mining_area_id)
        .bind(fixture.ai_robot_id)
        .bind(rally_result_id),
    )
    .await;

    let config = server_config(pool.clone());
    let username = sqlx::query_scalar::<_, String>("SELECT username FROM User WHERE id = ?")
        .bind(fixture.user_id)
        .fetch_one(&pool)
        .await
        .expect("fixture username");
    let cookie = format_authenticated_cookie(fixture.user_id, &username);

    let mut query = HashMap::new();
    query.insert("rallyResultId".to_string(), rally_result_id.to_string());
    query.insert("returnTo".to_string(), "runId=1".to_string());

    let mining_results = route(
        &get_request_query("/miningResults", query.clone(), Some(&cookie)),
        &config,
    )
    .await;
    let mining_body = response_body(&mining_results);
    assert_eq!(mining_results.status, 200, "{mining_body}");
    assert!(
        mining_body.contains("rally-view-page"),
        "expected miningResults rally view:\n{mining_body}"
    );
    assert!(
        mining_body.contains("Rally replay"),
        "expected rally title:\n{mining_body}"
    );
    assert!(
        mining_body.contains("Source snapshot unavailable."),
        "legacy queues without executedSourceCode must not invent live source:\n{mining_body}"
    );
    assert!(
        !mining_body.contains(r#"id="rallySourceCode""#),
        "unavailable snapshot should omit source panel code:\n{mining_body}"
    );

    let activity = route(
        &get_request_query("/activity", query, Some(&cookie)),
        &config,
    )
    .await;
    let activity_body = response_body(&activity);
    assert_eq!(activity.status, 200, "{activity_body}");
    assert!(
        activity_body.contains("rally-view-page"),
        "expected activity rally view:\n{activity_body}"
    );

    let missing = route(
        &get_request(
            &format!("/miningResults?rallyResultId={}", rally_result_id + 9_999),
            Some(&cookie),
        ),
        &config,
    )
    .await;
    assert_eq!(missing.status, 404);

    fixture.cleanup(&pool).await;
}
