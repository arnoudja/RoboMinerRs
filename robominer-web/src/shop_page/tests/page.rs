use std::path::PathBuf;

use crate::ServerConfig;
use crate::html::assert_html_contains;

use super::super::shop_page;
use super::fixtures::authenticated_request;

#[tokio::test(flavor = "current_thread")]
async fn shop_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = shop_page(&authenticated_request("/shop"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}
