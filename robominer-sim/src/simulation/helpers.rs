use robominer_program::ExecutableRunner;
use robominer_program::LANGUAGE_ORE_SLOTS;

use crate::animation::RecordedCpuStep;
use crate::ground::{Ground, ScanState};
use crate::robot::{ROBOT_ACTION_TYPE_SCAN, Robot, RobotAction};

/// Area-level values exposed to robot programs as `area.*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimulationAreaConfig {
    pub container_tax: i32,
    pub depot_tax: i32,
    pub ore_target: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct AreaSnapshot {
    pub container_tax: i32,
    pub depot_tax: i32,
    pub starting_ore: [i32; LANGUAGE_ORE_SLOTS],
    pub mining_turns: i32,
    pub ore_target: i32,
}

pub(super) fn starting_ore_from_ground(ground: &Ground) -> [i32; LANGUAGE_ORE_SLOTS] {
    let mut totals = [0; LANGUAGE_ORE_SLOTS];
    for x in 0..ground.size_x() {
        for y in 0..ground.size_y() {
            let unit = ground.at(x, y);
            for (slot, total) in totals.iter_mut().enumerate() {
                *total += unit.ore_at(slot);
            }
        }
    }
    totals
}

pub(super) fn animation_action_index(
    action: RobotAction,
    robot: &Robot,
    scanned_this_cycle: bool,
) -> u8 {
    let scan_busy = scanned_this_cycle || matches!(robot.scan_state, ScanState::Scanning { .. });
    if scan_busy && matches!(action, RobotAction::Wait) {
        ROBOT_ACTION_TYPE_SCAN as u8
    } else {
        action.action_index() as u8
    }
}

/// Carry forward the last known statement highlight (and refreshed locals) for
/// pending multi-cycle motion or battery-idle cycles that produce no new CPU steps.
pub(super) fn sticky_cpu_highlight(
    previous: &RecordedCpuStep,
    runner: Option<&ExecutableRunner>,
) -> RecordedCpuStep {
    let mut sticky = previous.clone();
    sticky.result = None;
    if let Some(runner) = runner {
        sticky.variables = runner.runtime_variables_snapshot();
    }
    sticky
}
