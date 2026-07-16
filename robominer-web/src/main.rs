use std::collections::HashMap;
use std::env;
use std::io;
use std::net::TcpListener;
use std::path::{Path, PathBuf};

use clap::Parser;
use robominer_web::{ServerConfig, block_on_database, serve};

#[derive(Debug, Parser)]
#[command(name = "robominer-web")]
#[command(about = "Rust web host for RoboMiner")]
struct Cli {
    #[arg(long)]
    database_url: Option<String>,

    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let default_static_root = default_web_root();
    let (config_path, legacy_config) = load_legacy_config(&cli);
    let settings = web_settings(&legacy_config, &default_static_root);

    let session_secret =
        robominer_web::resolve_session_secret(settings.session_secret.as_deref(), &settings.host)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let session_ttl_secs = robominer_web::resolve_session_ttl_secs(
        settings.session_ttl_secs.as_deref(),
        settings.session_ttl_hours.as_deref(),
        robominer_db::config_value(&legacy_config, "sessionttlsecs"),
        robominer_db::config_value(&legacy_config, "sessionttlhours"),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    robominer_web::configure_session_secret(&session_secret);
    robominer_web::configure_secure_cookies(settings.secure_cookies);
    robominer_web::configure_session_ttl_secs(session_ttl_secs);

    let database_pool = connect_database(&cli, &legacy_config)?;
    let listener = TcpListener::bind(format!("{}:{}", settings.host, settings.port))?;
    eprintln!(
        "robominer-web listening on http://{} with static root {} (config {})",
        listener.local_addr()?,
        settings.static_root.display(),
        config_path.display()
    );

    serve(
        listener,
        ServerConfig {
            static_root: settings.static_root,
            database_pool,
            allow_signup: settings.allow_signup,
            trust_proxy: settings.trust_proxy,
        },
    )
}

struct WebSettings {
    host: String,
    port: String,
    static_root: PathBuf,
    session_secret: Option<String>,
    session_ttl_secs: Option<String>,
    session_ttl_hours: Option<String>,
    secure_cookies: bool,
    allow_signup: bool,
    trust_proxy: bool,
}

fn load_legacy_config(cli: &Cli) -> (PathBuf, HashMap<String, String>) {
    match robominer_db::load_legacy_config(cli.config.clone(), "robominer-web") {
        Ok((config_path, config)) => (config_path, config),
        Err(robominer_db::ConfigError::MissingConfigFile) => {
            (PathBuf::from("<none>"), HashMap::new())
        }
        Err(error) => {
            eprintln!("failed to load config: {error}");
            std::process::exit(1);
        }
    }
}

fn web_settings(config: &HashMap<String, String>, default_static_root: &Path) -> WebSettings {
    WebSettings {
        host: env::var("HOST")
            .ok()
            .or_else(|| robominer_db::config_value(config, "host").map(str::to_owned))
            .unwrap_or_else(|| "127.0.0.1".to_string()),
        port: env::var("PORT")
            .ok()
            .or_else(|| robominer_db::config_value(config, "port").map(str::to_owned))
            .unwrap_or_else(|| "8080".to_string()),
        static_root: env::var("ROBOMINER_WEB_ROOT")
            .map(PathBuf::from)
            .ok()
            .or_else(|| robominer_db::config_value(config, "webroot").map(PathBuf::from))
            .unwrap_or_else(|| default_static_root.to_path_buf()),
        session_secret: env::var("ROBOMINER_SESSION_SECRET")
            .ok()
            .or_else(|| robominer_db::config_value(config, "sessionsecret").map(str::to_owned)),
        session_ttl_secs: env::var("ROBOMINER_SESSION_TTL_SECS").ok(),
        session_ttl_hours: env::var("ROBOMINER_SESSION_TTL_HOURS").ok(),
        secure_cookies: parse_bool_setting(
            env::var("ROBOMINER_SECURE_COOKIES").ok().as_deref(),
            robominer_db::config_value(config, "securecookies"),
        ),
        allow_signup: parse_bool_default_true(
            env::var("ROBOMINER_ALLOW_SIGNUP").ok().as_deref(),
            robominer_db::config_value(config, "allowsignup"),
        ),
        trust_proxy: parse_bool_setting(
            env::var("ROBOMINER_TRUST_PROXY").ok().as_deref(),
            robominer_db::config_value(config, "trustproxy"),
        ),
    }
}

fn parse_bool_default_true(env_value: Option<&str>, config_value: Option<&str>) -> bool {
    let value = env_value.or(config_value);
    match value {
        None => true,
        Some(value) => matches!(
            value.trim(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        ),
    }
}

fn parse_bool_setting(env_value: Option<&str>, config_value: Option<&str>) -> bool {
    if let Some(value) = env_value {
        return matches!(
            value.trim(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        );
    }

    config_value.is_some_and(|value| {
        matches!(
            value.trim(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )
    })
}

fn connect_database(
    cli: &Cli,
    legacy_config: &HashMap<String, String>,
) -> io::Result<Option<robominer_db::MySqlPool>> {
    let database_url = match robominer_db::resolve_database_url(
        cli.database_url.clone(),
        cli.config.clone(),
        "robominer-web",
    ) {
        Ok(url) => url,
        Err(robominer_db::ConfigError::MissingConfigFile) => return Ok(None),
        Err(error) => return Err(io::Error::other(error.to_string())),
    };

    let max_connections = robominer_db::resolve_max_connections(
        env::var("ROBOMINER_DB_MAX_CONNECTIONS").ok().as_deref(),
        robominer_db::config_value(legacy_config, "dbmaxconnections"),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;

    let pool = block_on_database(robominer_db::connect_with_max_connections(
        &database_url,
        max_connections,
    ))
    .map_err(|error| io::Error::other(format!("failed to connect to database: {error}")))?;

    Ok(Some(pool))
}

fn default_web_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("static")
}
