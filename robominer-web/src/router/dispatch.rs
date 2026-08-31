use crate::Request;
use crate::routes::AppRoute;
use crate::{
    Response, ServerConfig, account_page, achievements_page, auth_pages, edit_code_page,
    help_pages, leaderboard_page, login_redirect, mining_area_overview_page, mining_queue_page,
    mining_results_page, query_i64, rally_pages, request_user_id, robot_page, robot_stats_page,
    shop_page, static_files,
};

pub(super) async fn dispatch(request: &Request, config: &ServerConfig) -> Response {
    // Auth policy (by path family):
    // - Public: /health, /login|/signup|/logoff, /help*, /activity (read)
    // - Login required (+ CSRF on POST): shop, mining queue, robot, edit code,
    //   account, achievements, mining results, leaderboard (read), area overview
    // - Mining wallet claims: background worker via robominer-engine (rally rallies / mining claim-all)
    if !matches!(request.method.as_str(), "GET" | "HEAD" | "POST") {
        return Response::method_not_allowed();
    }

    if request.path == "/" {
        return if request_user_id(request).is_some() {
            Response::redirect(AppRoute::MiningQueue.href())
        } else {
            login_redirect(request)
        };
    }

    match AppRoute::from_path(&request.path) {
        Some(AppRoute::Achievements) => achievements_page::achievements_page(request, config).await,
        Some(AppRoute::Account) => account_page::account_page(request, config).await,
        Some(AppRoute::Activity) => rally_pages::activity_page(request, config).await,
        Some(AppRoute::EditCode) => edit_code_page::edit_code_page(request, config).await,
        Some(AppRoute::Help) => {
            help_pages::help_page(request, config, request.query.contains_key("welcome")).await
        }
        Some(AppRoute::HelpTutorial) => {
            help_pages::help_text_page(
                request,
                config,
                AppRoute::HelpTutorial.href(),
                query_i64(request, "step"),
            )
            .await
        }
        Some(AppRoute::HelpProgramTips) => {
            help_pages::help_text_page(request, config, AppRoute::HelpProgramTips.href(), None)
                .await
        }
        Some(AppRoute::HelpRobotProgram) => {
            help_pages::help_text_page(request, config, AppRoute::HelpRobotProgram.href(), None)
                .await
        }
        Some(AppRoute::HelpMechanics) => {
            help_pages::help_text_page(request, config, AppRoute::HelpMechanics.href(), None).await
        }
        Some(AppRoute::Leaderboard) => leaderboard_page::leaderboard_page(request, config).await,
        Some(AppRoute::Login) => auth_pages::login_page(request, config).await,
        Some(AppRoute::Logoff) => auth_pages::logoff_page(request, config).await,
        Some(AppRoute::MiningQueue) => mining_queue_page::mining_queue_page(request, config).await,
        Some(AppRoute::MiningResults) => {
            mining_results_page::mining_results_page(request, config).await
        }
        Some(AppRoute::MiningAreaOverview) => {
            mining_area_overview_page::mining_area_overview_page(request, config).await
        }
        Some(AppRoute::Robot) => robot_page::robot_page(request, config).await,
        Some(AppRoute::RobotStats) => robot_stats_page::robot_stats_page(request, config).await,
        Some(AppRoute::Shop) => shop_page::shop_page(request, config).await,
        None => static_files::static_response(&request.path, &config.static_root, request).await,
    }
}
