use crate::Request;
use crate::routes::AppRoute;
use crate::{
    Response, ServerConfig, account_page, achievements_page, auth_pages, edit_code_page,
    help_pages, leaderboard_page, mining_area_overview_page, mining_queue_page,
    mining_results_page, query_i64, rally_pages, robot_page, robot_stats_page, shop_page,
    static_files,
};

use super::route_policy::{enforce_policy, require_public_read, require_session};

pub(super) async fn dispatch(request: &Request, config: &ServerConfig) -> Response {
    if !matches!(request.method.as_str(), "GET" | "HEAD" | "POST") {
        return Response::method_not_allowed();
    }

    if request.path == "/" {
        return super::root_redirect(request).await;
    }

    let Some(route) = AppRoute::from_path(&request.path) else {
        return static_files::static_response(&request.path, &config.static_root, request).await;
    };

    match route {
        AppRoute::Achievements => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Achievements require ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => achievements_page::achievements_page(request, config, session).await,
                Err(response) => response,
            }
        }
        AppRoute::Account => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Account requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => account_page::account_page(request, config, session).await,
                Err(response) => response,
            }
        }
        AppRoute::Activity => {
            match require_public_read(enforce_policy(
                request,
                config,
                route.policy(),
                "Activity requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok((user_id, pool)) => {
                    rally_pages::activity_page(request, config, user_id, pool).await
                }
                Err(response) => response,
            }
        }
        AppRoute::EditCode => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Edit code requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => edit_code_page::edit_code_page(request, config, session).await,
                Err(response) => response,
            }
        }
        AppRoute::Help => {
            help_pages::help_page(request, config, request.query.contains_key("welcome")).await
        }
        AppRoute::HelpTutorial => {
            help_pages::help_text_page(
                request,
                config,
                AppRoute::HelpTutorial.href(),
                query_i64(request, "step"),
            )
            .await
        }
        AppRoute::HelpProgramTips => {
            help_pages::help_text_page(request, config, AppRoute::HelpProgramTips.href(), None)
                .await
        }
        AppRoute::HelpRobotProgram => {
            help_pages::help_text_page(request, config, AppRoute::HelpRobotProgram.href(), None)
                .await
        }
        AppRoute::HelpMechanics => {
            help_pages::help_text_page(request, config, AppRoute::HelpMechanics.href(), None).await
        }
        AppRoute::Leaderboard => {
            match require_public_read(enforce_policy(
                request,
                config,
                route.policy(),
                "Leaderboard requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok((user_id, pool)) => {
                    leaderboard_page::leaderboard_page(request, config, user_id, pool).await
                }
                Err(response) => response,
            }
        }
        AppRoute::Login => auth_pages::login_page(request, config).await,
        AppRoute::Logoff => auth_pages::logoff_page(request, config).await,
        AppRoute::MiningQueue => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Mining queue requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => mining_queue_page::mining_queue_page(request, config, session).await,
                Err(response) => response,
            }
        }
        AppRoute::MiningResults => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Mining results require ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => {
                    mining_results_page::mining_results_page(request, config, session).await
                }
                Err(response) => response,
            }
        }
        AppRoute::MiningAreaOverview => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Mining area overview requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => {
                    mining_area_overview_page::mining_area_overview_page(request, config, session)
                        .await
                }
                Err(response) => response,
            }
        }
        AppRoute::Robot => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Robot workshop requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => robot_page::robot_page(request, config, session).await,
                Err(response) => response,
            }
        }
        AppRoute::RobotStats => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Robot stats require ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => robot_stats_page::robot_stats_page(request, config, session).await,
                Err(response) => response,
            }
        }
        AppRoute::Shop => {
            match require_session(enforce_policy(
                request,
                config,
                route.policy(),
                "Shop requires ROBOMINER_DATABASE_URL to be configured",
            )) {
                Ok(session) => shop_page::shop_page(request, config, session).await,
                Err(response) => response,
            }
        }
    }
}
