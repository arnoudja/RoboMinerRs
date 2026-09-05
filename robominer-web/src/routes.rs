//! Canonical app route paths, optional legacy HTML aliases, and auth policy.

/// Access policy enforced by the router before page handlers run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutePolicy {
    /// No session required (login, help).
    Public,
    /// Readable without login; optional session enriches HUD and viewer context.
    PublicRead,
    /// Login required; POST mutations validate CSRF and rate limits when enabled.
    SessionRequired { csrf_on_post: bool },
}

/// Application page routes with stable camelCase URLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppRoute {
    Achievements,
    Account,
    Activity,
    EditCode,
    Help,
    HelpTutorial,
    HelpProgramTips,
    HelpRobotProgram,
    HelpMechanics,
    Leaderboard,
    Login,
    Logoff,
    MiningQueue,
    MiningResults,
    MiningAreaOverview,
    Robot,
    RobotStats,
    Shop,
}

impl AppRoute {
    pub const ALL: &[AppRoute] = &[
        Self::Achievements,
        Self::Account,
        Self::Activity,
        Self::EditCode,
        Self::Help,
        Self::HelpTutorial,
        Self::HelpProgramTips,
        Self::HelpRobotProgram,
        Self::HelpMechanics,
        Self::Leaderboard,
        Self::Login,
        Self::Logoff,
        Self::MiningQueue,
        Self::MiningResults,
        Self::MiningAreaOverview,
        Self::Robot,
        Self::RobotStats,
        Self::Shop,
    ];

    /// Relative camelCase href for HTML links (no leading slash).
    pub const fn href(self) -> &'static str {
        match self {
            Self::Achievements => "achievements",
            Self::Account => "account",
            Self::Activity => "activity",
            Self::EditCode => "editCode",
            Self::Help => "help",
            Self::HelpTutorial => "helpTutorial",
            Self::HelpProgramTips => "helpProgramTips",
            Self::HelpRobotProgram => "helpRobotProgram",
            Self::HelpMechanics => "helpMechanics",
            Self::Leaderboard => "leaderboard",
            Self::Login => "login",
            Self::Logoff => "logoff",
            Self::MiningQueue => "miningQueue",
            Self::MiningResults => "miningResults",
            Self::MiningAreaOverview => "miningAreaOverview",
            Self::Robot => "robot",
            Self::RobotStats => "robotStats",
            Self::Shop => "shop",
        }
    }

    /// Absolute canonical path (`/miningQueue`).
    pub const fn path(self) -> &'static str {
        match self {
            Self::Achievements => "/achievements",
            Self::Account => "/account",
            Self::Activity => "/activity",
            Self::EditCode => "/editCode",
            Self::Help => "/help",
            Self::HelpTutorial => "/helpTutorial",
            Self::HelpProgramTips => "/helpProgramTips",
            Self::HelpRobotProgram => "/helpRobotProgram",
            Self::HelpMechanics => "/helpMechanics",
            Self::Leaderboard => "/leaderboard",
            Self::Login => "/login",
            Self::Logoff => "/logoff",
            Self::MiningQueue => "/miningQueue",
            Self::MiningResults => "/miningResults",
            Self::MiningAreaOverview => "/miningAreaOverview",
            Self::Robot => "/robot",
            Self::RobotStats => "/robotStats",
            Self::Shop => "/shop",
        }
    }

    /// Extra absolute path aliases (legacy HTML filenames).
    pub const fn extra_aliases(self) -> &'static [&'static str] {
        match self {
            Self::HelpTutorial => &["/help_tutorial.html"],
            Self::HelpProgramTips => &["/help_programtips.html"],
            Self::HelpRobotProgram => &["/help_robotprogram.html"],
            Self::HelpMechanics => &["/help_mechanics.html"],
            _ => &[],
        }
    }

    /// True when `path` is the canonical path or an extra alias.
    pub fn matches(self, path: &str) -> bool {
        path == self.path() || self.extra_aliases().contains(&path)
    }

    pub fn from_path(path: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|route| route.matches(path))
    }

    /// Auth and CSRF policy for this route (see [`RoutePolicy`]).
    pub const fn policy(self) -> RoutePolicy {
        match self {
            Self::Login | Self::Logoff => RoutePolicy::Public,
            Self::Help
            | Self::HelpTutorial
            | Self::HelpProgramTips
            | Self::HelpRobotProgram
            | Self::HelpMechanics => RoutePolicy::Public,
            Self::Activity | Self::Leaderboard => RoutePolicy::PublicRead,
            Self::Achievements
            | Self::Account
            | Self::EditCode
            | Self::MiningQueue
            | Self::MiningResults
            | Self::MiningAreaOverview
            | Self::Robot
            | Self::RobotStats
            | Self::Shop => RoutePolicy::SessionRequired { csrf_on_post: true },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppRoute;

    #[test]
    fn each_route_path_and_aliases_resolve() {
        for route in AppRoute::ALL {
            assert_eq!(
                AppRoute::from_path(route.path()),
                Some(*route),
                "canonical path should resolve for {:?}",
                route
            );
            for alias in route.extra_aliases() {
                assert_eq!(
                    AppRoute::from_path(alias),
                    Some(*route),
                    "extra alias {alias} should resolve for {:?}",
                    route
                );
            }
            assert!(route.matches(route.path()));
            assert_eq!(format!("/{}", route.href()), route.path());
        }
    }

    #[test]
    fn pascal_case_paths_are_not_aliases() {
        for route in AppRoute::ALL {
            let href = route.href();
            let Some(first) = href.chars().next() else {
                continue;
            };
            let pascal = format!(
                "/{}{}",
                first.to_ascii_uppercase(),
                href.chars().skip(1).collect::<String>()
            );
            assert_ne!(pascal, route.path());
            assert!(
                !route.matches(&pascal),
                "{pascal} must not match {:?}",
                route
            );
            assert_eq!(
                AppRoute::from_path(&pascal),
                None,
                "{pascal} must not resolve"
            );
        }
    }

    #[test]
    fn unknown_paths_do_not_resolve() {
        assert_eq!(AppRoute::from_path("/notARealPage"), None);
        assert_eq!(AppRoute::from_path("/"), None);
    }

    #[test]
    fn each_route_has_expected_policy() {
        use super::RoutePolicy;

        let public = [
            AppRoute::Login,
            AppRoute::Logoff,
            AppRoute::Help,
            AppRoute::HelpTutorial,
            AppRoute::HelpProgramTips,
            AppRoute::HelpRobotProgram,
            AppRoute::HelpMechanics,
        ];
        let public_read = [AppRoute::Activity, AppRoute::Leaderboard];
        let session = [
            AppRoute::Achievements,
            AppRoute::Account,
            AppRoute::EditCode,
            AppRoute::MiningQueue,
            AppRoute::MiningResults,
            AppRoute::MiningAreaOverview,
            AppRoute::Robot,
            AppRoute::RobotStats,
            AppRoute::Shop,
        ];

        for route in public {
            assert_eq!(route.policy(), RoutePolicy::Public, "{route:?}");
        }
        for route in public_read {
            assert_eq!(route.policy(), RoutePolicy::PublicRead, "{route:?}");
        }
        for route in session {
            assert_eq!(
                route.policy(),
                RoutePolicy::SessionRequired { csrf_on_post: true },
                "{route:?}"
            );
        }
    }
}
