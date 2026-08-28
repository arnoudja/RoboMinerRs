use std::path::PathBuf;

use crate::ServerConfig;
use crate::html::assert_html_contains;

use super::super::edit_code_page;
use super::fixtures::authenticated_request;

#[tokio::test(flavor = "current_thread")]
async fn edit_code_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = edit_code_page(&authenticated_request("/editCode"), &config).await;
    let body = response.body_utf8();

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}
