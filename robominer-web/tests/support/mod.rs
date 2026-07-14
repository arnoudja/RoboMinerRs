#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::OnceLock;

use robominer_db::MySqlPool;
use robominer_test_support::{WebSmokeDbFixture, web_smoke_prefix};
use robominer_web::test_support::{
    Request, Response, ServerConfig, configure_session_secret, route,
};

static SESSION_CONFIGURED: OnceLock<()> = OnceLock::new();

pub fn ensure_session_configured() {
    SESSION_CONFIGURED.get_or_init(|| {
        configure_session_secret("robominer-web-integration-test-secret");
    });
}

pub fn server_config(pool: MySqlPool) -> ServerConfig {
    ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: Some(pool),
        allow_signup: true,
    }
}

pub use robominer_test_support::unique_prefix;

pub fn get_request(path: &str, cookie: Option<&str>) -> Request {
    request_with_form("GET", path, HashMap::new(), HashMap::new(), cookie)
}

pub fn post_request(path: &str, form: HashMap<String, String>, cookie: Option<&str>) -> Request {
    request_with_form("POST", path, HashMap::new(), form, cookie)
}

pub fn get_request_query(
    path: &str,
    query: HashMap<String, String>,
    cookie: Option<&str>,
) -> Request {
    request_with_form("GET", path, query, HashMap::new(), cookie)
}

pub fn post_request_query(
    path: &str,
    query: HashMap<String, String>,
    form: HashMap<String, String>,
    cookie: Option<&str>,
) -> Request {
    request_with_form("POST", path, query, form, cookie)
}

fn request_with_form(
    method: &str,
    path: &str,
    query: HashMap<String, String>,
    form: HashMap<String, String>,
    cookie: Option<&str>,
) -> Request {
    let mut headers = HashMap::new();
    if let Some(cookie) = cookie {
        headers.insert("cookie".to_string(), cookie.to_string());
    }

    let form_values = form
        .iter()
        .map(|(name, value)| (name.clone(), vec![value.clone()]))
        .collect();

    Request {
        method: method.to_string(),
        path: path.to_string(),
        query,
        form,
        form_values,
        headers,
    }
}

pub fn response_body(response: &Response) -> String {
    String::from_utf8(response.body.clone()).expect("response body should be utf-8")
}

pub fn cookie_header(response: &Response) -> String {
    response
        .headers
        .iter()
        .filter(|(name, _)| *name == "Set-Cookie")
        .map(|(_, value)| value.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

pub struct WebSmokeFixture {
    pub db: WebSmokeDbFixture,
    pub username: String,
    pub password: String,
}

impl Deref for WebSmokeFixture {
    type Target = WebSmokeDbFixture;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl WebSmokeFixture {
    pub async fn create(pool: &MySqlPool) -> Self {
        let prefix = web_smoke_prefix();
        let username = format!("{prefix}-user");
        let password = "test-password-1".to_string();
        let user_id =
            create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
        let db = WebSmokeDbFixture::create(pool, user_id, &prefix).await;

        Self {
            db,
            username,
            password,
        }
    }

    pub fn login(&self, config: &ServerConfig) -> Response {
        login_with_credentials(config, &self.username, &self.password)
    }

    pub fn mining_queue_page(&self, config: &ServerConfig, cookie: &str) -> Response {
        route(&get_request("/miningQueue", Some(cookie)), config)
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        self.db.cleanup(pool).await;
    }
}

pub fn login_with_credentials(config: &ServerConfig, username: &str, password: &str) -> Response {
    let mut form = HashMap::new();
    form.insert("loginName".to_string(), username.to_string());
    form.insert("password".to_string(), password.to_string());
    route(&post_request("/login", form, None), config)
}

pub fn create_user_via_engine(username: &str, email: &str, password: &str) -> i64 {
    let database_url = std::env::var("ROBOMINER_DATABASE_URL")
        .expect("ROBOMINER_DATABASE_URL must be set for web DB smoke tests");
    let engine_bin = std::env::var("CARGO_BIN_EXE_robominer-engine").unwrap_or_else(|_| {
        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| format!("{}/../target", env!("CARGO_MANIFEST_DIR")));
        format!("{target_dir}/debug/robominer-engine")
    });
    let output = std::process::Command::new(engine_bin)
        .arg("--database-url")
        .arg(&database_url)
        .arg("create-user")
        .arg("--username")
        .arg(username)
        .arg("--email")
        .arg(email)
        .arg("--password")
        .arg(password)
        .output()
        .expect("failed to execute robominer-engine create-user");

    assert!(
        output.status.success(),
        "create-user failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("create-user stdout should be utf-8")
        .trim()
        .parse()
        .expect("create-user should return the new user id")
}
