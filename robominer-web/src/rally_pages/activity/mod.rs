use super::{
    ACTIVITY_RALLY_MAX_AREAS, ActivityFeedQuery, ActivityPageState, ActivityRallyFilter,
    RallyViewBackLink,
};
use crate::{
    Request, Response, ServerConfig, block_on_database, query_i64, request_user_id,
    session_username,
};

pub(super) mod render;

pub fn activity_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Activity requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    if let Some(rally_result_id) = query_i64(request, "rallyResultId") {
        let user_id = request_user_id(request).unwrap_or(0);
        let feed_query = ActivityFeedQuery::from_request(request);
        let result = block_on_database(super::view::load_rally_view_state(
            pool,
            user_id,
            rally_result_id,
            false,
        ));

        if let Ok(Some(state)) = result {
            return Response::html(super::view::render::render_rally_view_page(
                session_username(request),
                crate::app_shell::hud_markup(request, config).as_deref(),
                &state,
                Some(RallyViewBackLink::Activity(feed_query)),
            ));
        }
    }

    let user_id = request_user_id(request).unwrap_or(0);
    let feed_query = ActivityFeedQuery::from_request(request);
    let result = block_on_database(load_activity_state(pool, user_id, feed_query));

    match result {
        Ok(state) => Response::html(render::render_activity_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config).as_deref(),
            &state,
            feed_query,
        )),
        Err(error) => Response::service_unavailable(format!("Unable to load activity: {error}")),
    }
}

async fn load_activity_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    feed_query: ActivityFeedQuery,
) -> Result<ActivityPageState, robominer_domain::DomainError> {
    let feed_user_id = match feed_query.filter {
        ActivityRallyFilter::Mine if user_id > 0 => Some(user_id),
        ActivityRallyFilter::Mine => None,
        ActivityRallyFilter::All => None,
    };
    let (recent_rallies, has_more_rallies) =
        if feed_query.filter == ActivityRallyFilter::Mine && user_id <= 0 {
            (Vec::new(), false)
        } else {
            robominer_domain::list_activity_recent_rally_feed(
                pool,
                feed_user_id,
                feed_query.area_id,
                feed_query.limit,
            )
            .await?
        };
    let mining_queue_ids: Vec<i64> = recent_rallies
        .iter()
        .map(|rally| rally.mining_queue_id)
        .collect();
    let participants =
        robominer_domain::list_activity_rally_participants_for_queues(pool, &mining_queue_ids)
            .await?;
    let rally_areas =
        robominer_domain::list_activity_rally_area_options(pool, ACTIVITY_RALLY_MAX_AREAS).await?;
    let (queue_items, asset_summary) = if user_id > 0 {
        (
            robominer_domain::list_mining_queue_page_items(pool, user_id).await?,
            Some(robominer_domain::load_user_asset_summary(pool, user_id).await?),
        )
    } else {
        (Vec::new(), None)
    };

    Ok(ActivityPageState {
        recent_users: robominer_domain::list_activity_recent_users(pool, 5).await?,
        recent_rallies,
        participants,
        rally_areas,
        has_more_rallies,
        queue_items,
        asset_summary,
    })
}
