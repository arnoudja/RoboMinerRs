use crate::Request;
use crate::Response;
use crate::session::{self, session_cookie_name};

/// How to treat the request session after checking `User.sessionVersion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SessionStrip {
    /// Session matches DB (or no session cookie).
    Keep,
    /// Version mismatch / unknown user: strip request cookie and clear via Set-Cookie.
    InvalidatePermanently,
    /// DB lookup failed: strip request auth for this request only (do not clear cookie).
    TreatAsAnonymous,
}

/// Map a session-version lookup to a strip action. `Err` means the DB was unreachable.
pub(super) fn session_strip_for_version_lookup(
    lookup: Result<Option<i32>, ()>,
    session_version: i32,
) -> SessionStrip {
    match lookup {
        Err(()) => SessionStrip::TreatAsAnonymous,
        Ok(current) if current == Some(session_version) => SessionStrip::Keep,
        Ok(_) => SessionStrip::InvalidatePermanently,
    }
}

pub(super) async fn strip_stale_session_cookie(
    request: &mut Request,
    pool: &robominer_db::MySqlPool,
) -> SessionStrip {
    let Some(session) = session::session_from_request(request) else {
        return SessionStrip::Keep;
    };

    let lookup = match robominer_db::get_user_session_version(pool, session.user_id).await {
        Ok(version) => Ok(version),
        Err(error) => {
            tracing::warn!(
                user_id = session.user_id,
                error = %error,
                "session version lookup failed; treating request as anonymous"
            );
            Err(())
        }
    };
    let action = session_strip_for_version_lookup(lookup, session.session_version);
    if matches!(
        action,
        SessionStrip::InvalidatePermanently | SessionStrip::TreatAsAnonymous
    ) && let Some(cookies) = request.headers.get_mut("cookie")
    {
        *cookies = strip_named_cookie(cookies, session_cookie_name());
    }
    action
}

fn strip_named_cookie(cookies: &str, name: &str) -> String {
    cookies
        .split(';')
        .filter_map(|cookie| {
            let trimmed = cookie.trim();
            if trimmed.is_empty() {
                return None;
            }
            let cookie_name = trimmed.split_once('=').map(|(n, _)| n).unwrap_or(trimmed);
            if cookie_name == name {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pub(super) fn clear_stale_session_cookies(response: Response) -> Response {
    let response = session::with_set_cookies(response, session::session_clear_cookie_headers())
        .with_header(
            "Set-Cookie",
            format!(
                "robominer_user_id=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
                session::secure_cookie_suffix()
            ),
        );
    session::with_set_cookies(response, session::username_clear_cookie_headers())
}
