//! Route access policy enforcement before page handlers run.

use robominer_db::MySqlPool;

use crate::ServerConfig;
use crate::http::{Request, Response};
use crate::page_context::PageSession;
use crate::request_helpers::request_user_id;
use crate::routes::RoutePolicy;

/// Resolved access for a routed request.
pub(super) enum RouteAccess<'a> {
    Public,
    PublicRead {
        user_id: Option<i64>,
        pool: &'a MySqlPool,
    },
    Session(PageSession<'a>),
}

pub(super) fn enforce_policy<'a>(
    request: &Request,
    config: &'a ServerConfig,
    policy: RoutePolicy,
    missing_db_message: &str,
) -> Result<RouteAccess<'a>, Response> {
    match policy {
        RoutePolicy::Public => Ok(RouteAccess::Public),
        RoutePolicy::PublicRead => {
            let Some(pool) = config.database_pool.as_ref() else {
                return Err(Response::service_unavailable(missing_db_message));
            };
            Ok(RouteAccess::PublicRead {
                user_id: request_user_id(request),
                pool,
            })
        }
        RoutePolicy::SessionRequired { csrf_on_post } => {
            let session = PageSession::require(request, config, missing_db_message, csrf_on_post)?;
            Ok(RouteAccess::Session(session))
        }
    }
}

pub(super) fn require_session<'a>(
    access: Result<RouteAccess<'a>, Response>,
) -> Result<PageSession<'a>, Response> {
    match access {
        Ok(RouteAccess::Session(session)) => Ok(session),
        Err(response) => Err(response),
        Ok(other) => {
            tracing::error!(
                access = route_access_label(&other),
                "route policy mismatch: expected Session"
            );
            Err(Response::internal_error())
        }
    }
}

pub(super) fn require_public_read<'a>(
    access: Result<RouteAccess<'a>, Response>,
) -> Result<(Option<i64>, &'a MySqlPool), Response> {
    match access {
        Ok(RouteAccess::PublicRead { user_id, pool }) => Ok((user_id, pool)),
        Err(response) => Err(response),
        Ok(other) => {
            tracing::error!(
                access = route_access_label(&other),
                "route policy mismatch: expected PublicRead"
            );
            Err(Response::internal_error())
        }
    }
}

fn route_access_label(access: &RouteAccess<'_>) -> &'static str {
    match access {
        RouteAccess::Public => "Public",
        RouteAccess::PublicRead { .. } => "PublicRead",
        RouteAccess::Session(_) => "Session",
    }
}
