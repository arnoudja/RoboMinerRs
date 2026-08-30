use crate::Request;
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

    match request.path.as_str() {
        "/" => {
            if request_user_id(request).is_some() {
                Response::redirect("miningQueue")
            } else {
                login_redirect(request)
            }
        }
        "/achievements" | "/Achievements" => {
            achievements_page::achievements_page(request, config).await
        }
        "/account" | "/Account" => account_page::account_page(request, config).await,
        "/activity" | "/Activity" => rally_pages::activity_page(request, config).await,
        "/editCode" | "/EditCode" => edit_code_page::edit_code_page(request, config).await,
        "/help" | "/Help" => {
            help_pages::help_page(request, config, request.query.contains_key("welcome")).await
        }
        "/helpTutorial" | "/help_tutorial.html" => {
            help_pages::help_text_page(request, config, "helpTutorial", query_i64(request, "step"))
                .await
        }
        "/helpProgramTips" | "/help_programtips.html" => {
            help_pages::help_text_page(request, config, "helpProgramTips", None).await
        }
        "/helpRobotProgram" | "/help_robotprogram.html" => {
            help_pages::help_text_page(request, config, "helpRobotProgram", None).await
        }
        "/helpMechanics" | "/help_mechanics.html" => {
            help_pages::help_text_page(request, config, "helpMechanics", None).await
        }
        "/leaderboard" | "/Leaderboard" => {
            leaderboard_page::leaderboard_page(request, config).await
        }
        "/login" | "/Login" => auth_pages::login_page(request, config).await,
        "/logoff" | "/Logoff" => auth_pages::logoff_page(request, config).await,
        "/miningQueue" | "/MiningQueue" => {
            mining_queue_page::mining_queue_page(request, config).await
        }
        "/miningResults" | "/MiningResults" => {
            mining_results_page::mining_results_page(request, config).await
        }
        "/miningAreaOverview" | "/MiningAreaOverview" => {
            mining_area_overview_page::mining_area_overview_page(request, config).await
        }
        "/robot" | "/Robot" => robot_page::robot_page(request, config).await,
        "/robotStats" | "/RobotStats" => robot_stats_page::robot_stats_page(request, config).await,
        "/shop" | "/Shop" => shop_page::shop_page(request, config).await,
        _ => static_files::static_response(&request.path, &config.static_root, request).await,
    }
}
