//! Shared auth / CSRF / DB pool prologue for authenticated HTML pages.

use robominer_db::{ClaimedUserResults, MySqlPool};
use robominer_domain::DomainError;

use crate::ServerConfig;
use crate::app_shell;
use crate::csrf;
use crate::http::{Request, Response};
use crate::request_helpers::{login_redirect, request_user_id, session_username};

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
        let Some(pool) = config.database_pool.as_ref() else {
            return Err(Response::service_unavailable(missing_db_message));
        };
        Ok(Self { user_id, pool })
    }

    /// Login + pool only (no CSRF). For read-only pages that still need a session.
    pub(crate) fn require_read(
        request: &Request,
        config: &'a ServerConfig,
        missing_db_message: &str,
    ) -> Result<Self, Response> {
        let Some(user_id) = request_user_id(request) else {
            return Err(login_redirect(request));
        };
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

/// Claim pending mining rewards. Call from authenticated page loaders that show
/// claim banners or depend on an up-to-date wallet.
pub(crate) async fn claim_user_results(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<ClaimedUserResults, DomainError> {
    Ok(robominer_db::claim_user_results(pool, user_id).await?)
}

/// Map a domain load failure to an HTTP response.
pub(crate) fn page_load_error(page: &str, error: DomainError) -> Response {
    let message = format!("Unable to load {page}: {error}");
    match &error {
        DomainError::ReferencedAiRobotMissing { .. }
        | DomainError::ReferencedRobotPartMissing { .. }
        | DomainError::ReferencedQueueRobotMissing { .. }
        | DomainError::ReferencedPoolMiningAreaMissing { .. }
        | DomainError::ReferencedPoolRobotMissing { .. } => Response::not_found(),
        DomainError::InvalidRallyLoadout { .. }
        | DomainError::InvalidPoolLoadout { .. }
        | DomainError::InvalidMiningAreaSize { .. }
        | DomainError::InvalidMiningAreaOreSupply { .. }
        | DomainError::TooManyMiningAreaOreTypes { .. }
        | DomainError::RobotIdOutOfRange(_) => Response::bad_request(message),
        _ => Response::service_unavailable(message),
    }
}
