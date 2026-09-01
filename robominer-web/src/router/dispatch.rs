use crate::Request;
use crate::routes::AppRoute;
use crate::{
    Response, ServerConfig, account_page, achievements_page, auth_pages, edit_code_page,
    help_pages, leaderboard_page, mining_area_overview_page, mining_queue_page,
    mining_results_page, query_i64, rally_pages, robot_page, robot_stats_page, shop_page,
    static_files,
};

use super::route_policy::{RouteAccess, enforce_policy};

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
        AppRoute::Achievements => match enforce_policy(
            request,
            config,
            route.policy(),
            "Achievements require ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                achievements_page::achievements_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::Account => match enforce_policy(
            request,
            config,
            route.policy(),
            "Account requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                account_page::account_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::Activity => match enforce_policy(
            request,
            config,
            route.policy(),
            "Activity requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::PublicRead { user_id, pool }) => {
                rally_pages::activity_page(request, config, user_id, pool).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::EditCode => match enforce_policy(
            request,
            config,
            route.policy(),
            "Edit code requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                edit_code_page::edit_code_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
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
        AppRoute::Leaderboard => match enforce_policy(
            request,
            config,
            route.policy(),
            "Leaderboard requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::PublicRead { user_id, pool }) => {
                leaderboard_page::leaderboard_page(request, config, user_id, pool).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::Login => auth_pages::login_page(request, config).await,
        AppRoute::Logoff => auth_pages::logoff_page(request, config).await,
        AppRoute::MiningQueue => match enforce_policy(
            request,
            config,
            route.policy(),
            "Mining queue requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                mining_queue_page::mining_queue_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::MiningResults => match enforce_policy(
            request,
            config,
            route.policy(),
            "Mining results require ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                mining_results_page::mining_results_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::MiningAreaOverview => match enforce_policy(
            request,
            config,
            route.policy(),
            "Mining area overview requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                mining_area_overview_page::mining_area_overview_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::Robot => match enforce_policy(
            request,
            config,
            route.policy(),
            "Robot workshop requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                robot_page::robot_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::RobotStats => match enforce_policy(
            request,
            config,
            route.policy(),
            "Robot stats require ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                robot_stats_page::robot_stats_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
        AppRoute::Shop => match enforce_policy(
            request,
            config,
            route.policy(),
            "Shop requires ROBOMINER_DATABASE_URL to be configured",
        ) {
            Ok(RouteAccess::Session(session)) => {
                shop_page::shop_page(request, config, session).await
            }
            Err(response) => response,
            Ok(_) => Response::internal_error(),
        },
    }
}
