mod support;

use std::collections::HashMap;

use robominer_test_support::RobotApplyFixture;
use robominer_web::test_support::{Response, ServerConfig, route};
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, get_request_query,
    post_request_query, response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn robot_apply_post_persists_part_change_across_refresh() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping robot apply integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let database_url = std::env::var("ROBOMINER_DATABASE_URL").expect("database url");
    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");

    let prefix = unique_prefix("rust-web-robot-apply");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id = create_user_via_engine(
        &username,
        &format!("{prefix}@example.invalid"),
        &password,
    );
    let fixture = RobotApplyFixture::create(&pool, user_id, username, password).await;
    let config = server_config(pool.clone());

    let login_response = robot_apply_login(&fixture, &config);
    assert_eq!(login_response.status, 302, "login should redirect after success");
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("robotId".to_string(), fixture.robot_id.to_string());
    let apply_response = route(
        &post_request_query("/robot", query.clone(), fixture.apply_form(), Some(&cookie)),
        &config,
    );
    let apply_body = response_body(&apply_response);

    assert_eq!(apply_response.status, 200, "robot apply should render");
    assert!(
        apply_body.contains("Robot changes queued"),
        "expected apply success banner in response body:\n{apply_body}"
    );
    assert!(
        apply_body.contains(&fixture.spare_ore_container_name),
        "expected selected spare part in apply response body:\n{apply_body}"
    );

    fixture
        .assert_ore_container_id(&pool, fixture.spare_ore_container_id)
        .await;

    let refresh_response = route(
        &get_request_query("/robot", query, Some(&cookie)),
        &config,
    );
    let refresh_body = response_body(&refresh_response);

    assert_eq!(refresh_response.status, 200, "robot page refresh should render");
    assert!(
        refresh_body.contains(&format!(
            r#"value="{}" selected="selected""#,
            fixture.spare_ore_container_id
        )),
        "expected spare ore container to remain selected after refresh:\n{refresh_body}"
    );
    assert!(
        refresh_body.contains(&fixture.spare_ore_container_name),
        "expected spare ore container name after refresh:\n{refresh_body}"
    );

    fixture.cleanup(&pool).await;
}

fn robot_apply_login(fixture: &RobotApplyFixture, config: &ServerConfig) -> Response {
    let mut form = HashMap::new();
    form.insert("loginName".to_string(), fixture.username.clone());
    form.insert("password".to_string(), fixture.password.clone());
    route(&support::post_request("/login", form, None), config)
}
