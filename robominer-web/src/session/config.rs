use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub const DEFAULT_SESSION_TTL_HOURS: u64 = 24;
pub const DEFAULT_SESSION_TTL_SECS: u64 = DEFAULT_SESSION_TTL_HOURS * 60 * 60;
/// Upper bound for configured (non-remember-me) session TTL.
pub const MAX_SESSION_TTL_SECS: u64 = 30 * 24 * 60 * 60;
pub const DEFAULT_DEV_SESSION_SECRET: &str = "robominer-dev-session-secret-change-me";
/// Minimum length for configured session secrets (dev default already qualifies).
pub const MIN_SESSION_SECRET_LEN: usize = 32;

pub(super) static SECURE_COOKIES: AtomicBool = AtomicBool::new(false);
pub(super) static SESSION_TTL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_SESSION_TTL_SECS);

pub fn is_local_bind_host(host: &str) -> bool {
    matches!(host.trim(), "127.0.0.1" | "localhost" | "::1")
}

pub fn resolve_session_secret(
    configured: Option<&str>,
    bind_host: &str,
    allow_insecure_dev_secret: bool,
) -> Result<String, &'static str> {
    if let Some(secret) = configured
        .map(str::trim)
        .filter(|secret| !secret.is_empty())
    {
        if secret.len() < MIN_SESSION_SECRET_LEN {
            return Err(
                "ROBOMINER_SESSION_SECRET (or sessionsecret in config) must be at least 32 characters",
            );
        }
        return Ok(secret.to_string());
    }

    if is_local_bind_host(bind_host) && allow_insecure_dev_secret {
        tracing::warn!(
            "ROBOMINER_SESSION_SECRET is not set; using an insecure development default \
             (ROBOMINER_ALLOW_INSECURE_DEV_SECRET=1)"
        );
        return Ok(DEFAULT_DEV_SESSION_SECRET.to_string());
    }

    if is_local_bind_host(bind_host) {
        return Err(
            "ROBOMINER_SESSION_SECRET (or sessionsecret in config) is required, \
             or set ROBOMINER_ALLOW_INSECURE_DEV_SECRET=1 for local development only",
        );
    }

    Err(
        "ROBOMINER_SESSION_SECRET (or sessionsecret in config) is required when binding to a non-localhost address",
    )
}

/// Refuse `trust_proxy` unless the process binds loopback (proxy must own public traffic).
pub fn validate_trust_proxy_bind(host: &str, trust_proxy: bool) -> Result<(), &'static str> {
    if trust_proxy && !is_local_bind_host(host) {
        return Err(
            "trustproxy / ROBOMINER_TRUST_PROXY requires binding to 127.0.0.1, localhost, or ::1 \
             so client forwarding headers cannot be spoofed",
        );
    }
    Ok(())
}

/// Resolve the Secure cookie flag.
///
/// Explicit `securecookies` / env wins when set. Defaults **off** on loopback
/// so local HTTP keeps working. Non-loopback binds require Secure cookies
/// (stealable session cookies over cleartext LAN/WAN HTTP). When `trust_proxy`
/// is on (TLS terminated at a reverse proxy), Secure cookies are required —
/// refuse rather than silently serving stealable session cookies over any HTTP
/// hop to the proxy.
pub fn resolve_secure_cookies(
    configured: Option<bool>,
    bind_host: &str,
    trust_proxy: bool,
) -> Result<bool, &'static str> {
    let enabled = configured.unwrap_or(false);
    if trust_proxy && !enabled {
        return Err(
            "trustproxy / ROBOMINER_TRUST_PROXY requires securecookies 1 \
             (or ROBOMINER_SECURE_COOKIES=1) so session cookies are marked Secure behind TLS",
        );
    }
    if !enabled && !is_local_bind_host(bind_host) {
        return Err(
            "non-loopback bind requires securecookies 1 (or ROBOMINER_SECURE_COOKIES=1) \
             so session cookies are marked Secure",
        );
    }
    Ok(enabled)
}

pub fn configure_secure_cookies(enabled: bool) {
    SECURE_COOKIES.store(enabled, Ordering::Relaxed);
}

pub fn configure_session_ttl_secs(ttl_secs: u64) {
    SESSION_TTL_SECS.store(ttl_secs, Ordering::Relaxed);
}

pub fn resolve_session_ttl_secs(
    env_secs: Option<&str>,
    env_hours: Option<&str>,
    config_secs: Option<&str>,
    config_hours: Option<&str>,
) -> Result<u64, String> {
    if let Some(value) = env_secs {
        return parse_session_ttl_secs(value, "ROBOMINER_SESSION_TTL_SECS");
    }
    if let Some(value) = env_hours {
        return parse_session_ttl_hours(value, "ROBOMINER_SESSION_TTL_HOURS");
    }
    if let Some(value) = config_secs {
        return parse_session_ttl_secs(value, "sessionttlsecs");
    }
    if let Some(value) = config_hours {
        return parse_session_ttl_hours(value, "sessionttlhours");
    }
    Ok(DEFAULT_SESSION_TTL_SECS)
}

fn parse_session_ttl_secs(value: &str, name: &str) -> Result<u64, String> {
    let ttl_secs = value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("{name} must be a positive integer"))?;
    validate_session_ttl_secs(ttl_secs, name)
}

fn parse_session_ttl_hours(value: &str, name: &str) -> Result<u64, String> {
    let hours = value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("{name} must be a positive integer"))?;
    if hours == 0 {
        return Err(format!("{name} must be greater than 0"));
    }
    let ttl_secs = hours
        .checked_mul(60 * 60)
        .ok_or_else(|| format!("{name} is too large"))?;
    validate_session_ttl_secs(ttl_secs, name)
}

fn validate_session_ttl_secs(ttl_secs: u64, name: &str) -> Result<u64, String> {
    if ttl_secs == 0 {
        return Err(format!("{name} must be greater than 0"));
    }
    if ttl_secs > MAX_SESSION_TTL_SECS {
        return Err(format!(
            "{name} must be at most {MAX_SESSION_TTL_SECS} seconds (30 days)"
        ));
    }
    Ok(ttl_secs)
}

pub(super) fn session_ttl_secs(persistent: bool) -> u64 {
    if persistent {
        MAX_SESSION_TTL_SECS
    } else {
        SESSION_TTL_SECS.load(Ordering::Relaxed)
    }
}

pub(crate) fn secure_cookie_suffix() -> &'static str {
    if SECURE_COOKIES.load(Ordering::Relaxed) {
        "; Secure"
    } else {
        ""
    }
}
