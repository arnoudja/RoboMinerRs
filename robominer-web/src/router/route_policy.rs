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
