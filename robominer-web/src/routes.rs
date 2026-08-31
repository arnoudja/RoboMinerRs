//! Canonical app route paths and camelCase / PascalCase alias matching.

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

    /// Absolute PascalCase alias (`/MiningQueue`).
    pub const fn pascal_path(self) -> &'static str {
        match self {
            Self::Achievements => "/Achievements",
            Self::Account => "/Account",
            Self::Activity => "/Activity",
            Self::EditCode => "/EditCode",
            Self::Help => "/Help",
            Self::HelpTutorial => "/HelpTutorial",
            Self::HelpProgramTips => "/HelpProgramTips",
            Self::HelpRobotProgram => "/HelpRobotProgram",
            Self::HelpMechanics => "/HelpMechanics",
            Self::Leaderboard => "/Leaderboard",
            Self::Login => "/Login",
            Self::Logoff => "/Logoff",
            Self::MiningQueue => "/MiningQueue",
            Self::MiningResults => "/MiningResults",
            Self::MiningAreaOverview => "/MiningAreaOverview",
            Self::Robot => "/Robot",
            Self::RobotStats => "/RobotStats",
            Self::Shop => "/Shop",
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

    /// True when `path` is the canonical path, PascalCase alias, or an extra alias.
    pub fn matches(self, path: &str) -> bool {
        path == self.path() || path == self.pascal_path() || self.extra_aliases().contains(&path)
    }

    pub fn from_path(path: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|route| route.matches(path))
    }

    /// Canonical absolute path when `path` matches this route under any alias.
    pub fn canonicalize(path: &str) -> Option<&'static str> {
        Self::from_path(path).map(Self::path)
    }
}

#[cfg(test)]
mod tests {
    use super::AppRoute;

    #[test]
    fn each_route_path_and_aliases_canonicalize() {
        for route in AppRoute::ALL {
            assert_eq!(
                AppRoute::canonicalize(route.path()),
                Some(route.path()),
                "canonical path should resolve for {:?}",
                route
            );
            assert_eq!(
                AppRoute::canonicalize(route.pascal_path()),
                Some(route.path()),
                "PascalCase alias should resolve for {:?}",
                route
            );
            for alias in route.extra_aliases() {
                assert_eq!(
                    AppRoute::canonicalize(alias),
                    Some(route.path()),
                    "extra alias {alias} should resolve for {:?}",
                    route
                );
            }
            assert!(route.matches(route.path()));
            assert!(route.matches(route.pascal_path()));
            assert_eq!(format!("/{}", route.href()), route.path());
        }
    }

    #[test]
    fn unknown_paths_do_not_canonicalize() {
        assert_eq!(AppRoute::canonicalize("/notARealPage"), None);
        assert_eq!(AppRoute::canonicalize("/"), None);
    }
}
