#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HelpGuide {
    pub(crate) href: &'static str,
    pub(crate) title: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) tag: &'static str,
    pub(crate) body: &'static str,
}

mod content;
mod render;

pub(crate) const HELP_GUIDES: [HelpGuide; 4] = [
    HelpGuide {
        href: "helpTutorial",
        title: "Tutorial",
        summary: "Step-by-step first steps to keep your robot mining and upgrading.",
        tag: "Start here",
        body: "",
    },
    HelpGuide {
        href: "helpProgramTips",
        title: "Programming tips",
        summary: "Examples and patterns to mine more ore. Contains spoilers.",
        tag: "Tips",
        body: content::PROGRAM_TIPS_BODY,
    },
    HelpGuide {
        href: "helpRobotProgram",
        title: "Robot programming language",
        summary: "Syntax, statements, and language reference for robot programs.",
        tag: "Reference",
        body: content::ROBOT_PROGRAM_BODY,
    },
    HelpGuide {
        href: "helpMechanics",
        title: "Mechanics",
        summary: "What shop stats mean, how CPU vs physical actions work, and how scanning interacts with ore heaps.",
        tag: "Reference",
        body: content::MECHANICS_BODY,
    },
];

pub(crate) fn guide_by_href(href: &str) -> Option<&'static HelpGuide> {
    HELP_GUIDES
        .iter()
        .find(|guide| guide.href.eq_ignore_ascii_case(href))
}

pub(crate) use render::{
    render_help_article, render_help_index, render_page_help_hint, render_page_help_hint_line,
    welcome_banner_markup,
};
