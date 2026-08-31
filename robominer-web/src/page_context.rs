//! Shared auth / CSRF / DB pool prologue for authenticated HTML pages.

use std::fmt;

use robominer_db::MySqlPool;

use crate::ServerConfig;
use crate::app_shell;
use crate::csrf;
use crate::http::{Request, Response};
use crate::request_helpers::{login_redirect, request_user_id, session_username};

/// Database failure while loading an HTML page (or auth prologue that only does SQL).
///
/// Page read models should use this instead of [`robominer_domain::DomainError`], which is
/// reserved for loadout/simulation and other domain rule failures.
///
/// The underlying SQL error is intentionally opaque so callers cannot pattern-match on
/// `sqlx::Error` or read it as a public field; [`Display`] still surfaces the message for
/// operator logs.
#[derive(Debug)]
pub(crate) struct PageLoadError(PageLoadErrorKind);

#[derive(Debug)]
enum PageLoadErrorKind {
    Sql(sqlx::Error),
    Message(String),
}

impl From<sqlx::Error> for PageLoadError {
    fn from(error: sqlx::Error) -> Self {
        Self(PageLoadErrorKind::Sql(error))
    }
}

impl PageLoadError {
    /// Convert a domain façade failure that is only expected to be SQL-backed (e.g.
    /// program create/update) into a page-load error. Only [`DomainError::Database`]
    /// is accepted here; callers must handle other variants explicitly, since they
    /// are not expected on page-load paths and shouldn't be silently reinterpreted
    /// as SQL errors.
    pub(crate) fn from_database(
        error: robominer_domain::DomainError,
    ) -> Result<Self, robominer_domain::DomainError> {
        match error {
            robominer_domain::DomainError::Database(error) => {
                Ok(Self(PageLoadErrorKind::Message(error.to_string())))
            }
            other => Err(other),
        }
    }
}

impl fmt::Display for PageLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            PageLoadErrorKind::Sql(error) => error.fmt(f),
            PageLoadErrorKind::Message(message) => f.write_str(message),
        }
    }
}

/// Authenticated page session with a configured database pool.
pub(crate) struct PageSession<'a> {
    pub user_id: i64,
    pub pool: &'a MySqlPool,
}

impl<'a> PageSession<'a> {
    /// Require login, valid CSRF (for mutating methods), and a database pool.
    pub(crate) fn require(
        request: &Request,
        config: &'a ServerConfig,
        missing_db_message: &str,
    ) -> Result<Self, Response> {
        let Some(user_id) = request_user_id(request) else {
            return Err(login_redirect(request));
        };
        if let Some(response) = csrf::reject_invalid_csrf(request, user_id) {
            return Err(response);
        }
        if crate::request_helpers::is_post(request)
            && let Some(response) =
                crate::rate_limit::reject_rate_limited_mutation(request, user_id)
        {
            return Err(response);
        }
        let Some(pool) = config.database_pool.as_ref() else {
            return Err(Response::service_unavailable(missing_db_message));
        };
        Ok(Self { user_id, pool })
    }

    /// Render HTML with CSRF cookie refresh and HUD markup.
    pub(crate) async fn html_with_hud(
        &self,
        request: &Request,
        config: &ServerConfig,
        render: impl FnOnce(String, Option<&str>) -> String,
    ) -> Response {
        csrf::html_with_csrf(
            request,
            self.user_id,
            render(
                session_username(request),
                app_shell::hud_markup(request, config).await.as_deref(),
            ),
        )
    }

    /// Render HTML without CSRF rotation (read-only pages).
    pub(crate) async fn html_read_with_hud(
        &self,
        request: &Request,
        config: &ServerConfig,
        render: impl FnOnce(String, Option<&str>) -> String,
    ) -> Response {
        Response::html(render(
            session_username(request),
            app_shell::hud_markup(request, config).await.as_deref(),
        ))
    }
}

/// Run an authenticated page handler after [`PageSession::require`].
pub(crate) async fn with_session_page<'a, F, Fut>(
    request: &'a Request,
    config: &'a ServerConfig,
    missing_db_message: &str,
    handler: F,
) -> Response
where
    F: FnOnce(PageSession<'a>) -> Fut,
    Fut: std::future::Future<Output = Response>,
{
    match PageSession::require(request, config, missing_db_message) {
        Ok(session) => handler(session).await,
        Err(response) => response,
    }
}

/// Clamp a positive page-size query into `[default_limit, max_limit]`.
pub(crate) fn clamp_page_limit(raw: Option<i64>, default_limit: i64, max_limit: i64) -> i64 {
    raw.unwrap_or(default_limit).clamp(default_limit, max_limit)
}

/// Map a page-load database failure to an HTTP response.
///
/// Client bodies stay generic so SQL details are not leaked; the full error is
/// logged server-side for operators.
pub(crate) fn page_load_error(page: &str, error: PageLoadError) -> Response {
    tracing::error!(page, error = %error, "Unable to load page");
    Response::service_unavailable(format!("Unable to load {page}. Please try again shortly."))
}
