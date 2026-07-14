pub(super) const PROGRAM_TIPS_BODY: &str = include_str!("../../static/help/program_tips.html");
pub(super) const ROBOT_PROGRAM_BODY: &str = include_str!("../../static/help/robot_program.html");
pub(super) const MECHANICS_BODY: &str = include_str!("../../static/help/mechanics.html");
pub(super) const TUTORIAL_INTRO: &str = include_str!("../../static/help/tutorial_intro.html");

pub(super) struct TutorialStep {
    pub(super) title: &'static str,
    pub(super) body: &'static str,
}

pub(super) const TUTORIAL_STEPS: [TutorialStep; 5] = [
    TutorialStep {
        title: "Keep your robot busy",
        body: include_str!("../../static/help/tutorial_step_1.html"),
    },
    TutorialStep {
        title: "Claim achievements",
        body: include_str!("../../static/help/tutorial_step_2.html"),
    },
    TutorialStep {
        title: "Review mining results",
        body: include_str!("../../static/help/tutorial_step_3.html"),
    },
    TutorialStep {
        title: "Upgrade your robot",
        body: include_str!("../../static/help/tutorial_step_4.html"),
    },
    TutorialStep {
        title: "Improve your robot program",
        body: include_str!("../../static/help/tutorial_step_5.html"),
    },
];
