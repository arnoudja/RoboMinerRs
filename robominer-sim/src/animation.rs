use std::collections::BTreeMap;

use crate::MAX_ORE_TYPES;
use crate::animation_payload::{
    AnimationCpuStep, AnimationCpuStepResult, AnimationCpuStepResultKind, AnimationGround,
    AnimationGroundChange, AnimationGroundPosition, AnimationLocation, AnimationOreType,
    AnimationPayload, AnimationRobot, AnimationRobots,
};
use crate::ground::Ground;
use crate::position::Position;
use crate::robot::Robot;

/// Current on-disk / wire format for rally animation payloads stored in
/// `RallyResult.resultData`. Older executable JavaScript rows (`var myRobots = …`)
/// are no longer played by the web viewer.
///
/// Version 2 adds optional per-turn `cpu` arrays of instruction spans,
/// typed step results (`r`), and locals snapshots (`vs`).
pub const ANIMATION_PAYLOAD_VERSION: u32 = 2;

pub struct OreAnimationData {
    pub ore_id: i64,
    pub max_amount: i32,
}

/// One program CPU instruction within a turn (for replay stepping/highlight).
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedCpuStep {
    pub line: u16,
    pub start_col: u16,
    pub end_col: u16,
    pub result: Option<robominer_program::CpuStepResult>,
    pub variables: BTreeMap<String, robominer_program::CpuStepResult>,
}

impl RecordedCpuStep {
    pub fn from_span(span: robominer_program::SourceSpan) -> Option<Self> {
        if !span.is_known() {
            return None;
        }
        Some(Self {
            line: span.line,
            start_col: span.start_col,
            end_col: span.end_col,
            result: None,
            variables: BTreeMap::new(),
        })
    }

    /// True when this step has a token column range (matches [`SourceSpan::has_columns`]).
    pub fn has_columns(&self) -> bool {
        self.line != 0 && self.start_col > 0 && self.end_col > self.start_col
    }

    pub fn with_result(mut self, result: Option<robominer_program::CpuStepResult>) -> Self {
        self.result = result;
        self
    }

    pub fn with_variables(
        mut self,
        variables: BTreeMap<String, robominer_program::CpuStepResult>,
    ) -> Self {
        self.variables = variables;
        self
    }
}

fn animation_cpu_step_result(result: robominer_program::CpuStepResult) -> AnimationCpuStepResult {
    AnimationCpuStepResult {
        k: AnimationCpuStepResultKind::from(result.kind),
        v: result.value,
    }
}

impl From<RecordedCpuStep> for AnimationCpuStep {
    fn from(step: RecordedCpuStep) -> Self {
        let has_columns = step.has_columns();
        let mut entry = AnimationCpuStep {
            l: step.line,
            c: None,
            e: None,
            r: step.result.map(animation_cpu_step_result),
            vs: if step.variables.is_empty() {
                None
            } else {
                Some(
                    step.variables
                        .into_iter()
                        .map(|(name, result)| (name, animation_cpu_step_result(result)))
                        .collect(),
                )
            },
        };
        if has_columns {
            entry.c = Some(step.start_col);
            entry.e = Some(step.end_col);
        }
        entry
    }
}

/// Compact per-cycle status for stuck/idle diagnosis in the replay viewer.
/// Omitted from JSON when the robot is making normal progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobotCycleStatus {
    /// Individual battery / max_turns exhausted; no action this cycle.
    Battery,
    /// Waiting while a scan completes (paired with action index 0).
    Scan,
    /// CPU budget exhausted before an action was chosen.
    Cpu,
    /// `move(0)` / `rotate(0)` (or epsilon-equivalent) mapped to Wait.
    Zero,
    /// Non-zero motion requested but collapsed to Wait — no speed chunk could be issued
    /// (e.g. zero engine speed). Wire status remains `"motion"` for replay compatibility.
    NoChunk,
    /// Requested move ended at the start pose due to map bounds.
    Wall,
    /// Requested move ended at the start pose due to another robot.
    Robot,
    /// Explicit or residual Wait with no more specific cause.
    Wait,
}

impl RobotCycleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Battery => "battery",
            Self::Scan => "scan",
            Self::Cpu => "cpu",
            Self::Zero => "zero",
            Self::NoChunk => "motion",
            Self::Wall => "wall",
            Self::Robot => "robot",
            Self::Wait => "wait",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RobotAnimationStep {
    position: Position,
    ore: [i32; MAX_ORE_TYPES],
    depot: [i32; MAX_ORE_TYPES],
    time_fraction: f64,
    /// Optional action index for this cycle (`RobotAction::action_index`, or 0 for scan).
    /// Absent on the initial step and on legacy replays.
    action_index: Option<u8>,
    /// Optional 1-based source line when this cycle has no CPU micro-steps (sticky highlight).
    /// Serialized as `l`; omitted when `cpu_steps` is non-empty.
    source_line: Option<u16>,
    /// Optional stuck/idle reason for this cycle.
    status: Option<RobotCycleStatus>,
    /// Program CPU micro-steps for this cycle; serialized as `cpu` when non-empty.
    cpu_steps: Vec<RecordedCpuStep>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GroundAnimationStep {
    time: i32,
    ore: [i32; MAX_ORE_TYPES],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct AnimationRecorder {
    robot_steps: Vec<Vec<RobotAnimationStep>>,
    ground_changes: BTreeMap<(usize, usize), Vec<GroundAnimationStep>>,
}

impl AnimationRecorder {
    pub(crate) fn new(robot_count: usize) -> Self {
        Self {
            robot_steps: vec![Vec::new(); robot_count],
            ground_changes: BTreeMap::new(),
        }
    }

    pub(crate) fn record_initial_ground(&mut self, ground: &Ground) {
        for x in 0..ground.size_x() {
            for y in 0..ground.size_y() {
                let ore = *ground.at(x, y).ore();
                if ore.iter().any(|amount| *amount > 0) {
                    self.record_ground_change(x, y, 0, ore);
                }
            }
        }
    }

    pub(crate) fn record_ground_change(
        &mut self,
        x: usize,
        y: usize,
        time: i32,
        ore: [i32; MAX_ORE_TYPES],
    ) {
        self.ground_changes
            .entry((x, y))
            .or_default()
            .push(GroundAnimationStep { time, ore });
    }

    pub(crate) fn record_robot_step(
        &mut self,
        robot_index: usize,
        robot: &Robot,
        action_index: Option<u8>,
        source_line: Option<u16>,
        status: Option<RobotCycleStatus>,
        cpu_steps: Vec<RecordedCpuStep>,
    ) {
        self.robot_steps[robot_index].push(RobotAnimationStep {
            position: robot.position(),
            ore: *robot.ore(),
            depot: *robot.depot(),
            time_fraction: robot.time_fraction,
            action_index,
            source_line,
            status,
            cpu_steps,
        });
    }

    pub(crate) fn into_animation_payload(
        self,
        ground: &Ground,
        robots: &[Robot],
        ore_data: &[OreAnimationData],
    ) -> AnimationPayload {
        AnimationPayload {
            v: ANIMATION_PAYLOAD_VERSION,
            robots: robots_animation(&self.robot_steps, robots, ground.size_x(), ground.size_y()),
            ground: ground_animation(ground, &self.ground_changes),
            ore_types: ore_animation(ore_data),
        }
    }

    pub(crate) fn into_animation_data(
        self,
        ground: &Ground,
        robots: &[Robot],
        ore_data: &[OreAnimationData],
    ) -> String {
        self.into_animation_payload(ground, robots, ore_data)
            .to_embedded_json()
    }
}

/// Axis-aligned square on the map corner where this robot slot spawns.
/// Side length is `ceil(robot_size)` cells, anchored at the map corner.
fn depot_home_square(
    robot_index: usize,
    robot_size: f64,
    size_x: usize,
    size_y: usize,
) -> (usize, usize, usize) {
    let side = robot_size.ceil().max(1.0) as usize;
    let side = side.min(size_x.max(1)).min(size_y.max(1));
    let (x, y) = match robot_index {
        0 => (0, 0),
        1 => (0, size_y.saturating_sub(side)),
        2 => (size_x.saturating_sub(side), 0),
        3 => (size_x.saturating_sub(side), size_y.saturating_sub(side)),
        _ => (0, 0),
    };
    (x, y, side)
}

fn robots_animation(
    robot_steps: &[Vec<RobotAnimationStep>],
    robots: &[Robot],
    size_x: usize,
    size_y: usize,
) -> AnimationRobots {
    let mut robot_values = Vec::with_capacity(robot_steps.len());

    for (index, steps) in robot_steps.iter().enumerate() {
        let Some(first_step) = steps.first() else {
            continue;
        };
        let spec = robots[index].spec();
        let depot_capacity = robots[index].depot_capacity();
        let record_depot = depot_capacity.iter().take(3).any(|&cap| cap > 0);

        let mut robot = AnimationRobot {
            robotnr: index,
            x: legacy_float(first_step.position.x),
            y: legacy_float(first_step.position.y),
            o: first_step.position.orientation,
            ore_a: first_step.ore[0],
            ore_b: first_step.ore[1],
            ore_c: first_step.ore[2],
            size: legacy_float(spec.robot_size),
            maxore: spec.max_ore,
            maxturns: spec.max_turns,
            cpuspeed: spec.cpu_speed,
            depot_max_a: None,
            depot_max_b: None,
            depot_max_c: None,
            depot_a: None,
            depot_b: None,
            depot_c: None,
            home_x: None,
            home_y: None,
            home_size: None,
            locations: robot_locations(steps, record_depot),
        };

        if record_depot {
            let (home_x, home_y, home_size) =
                depot_home_square(index, spec.robot_size, size_x, size_y);
            robot.depot_max_a = Some(depot_capacity[0]);
            robot.depot_max_b = Some(depot_capacity[1]);
            robot.depot_max_c = Some(depot_capacity[2]);
            robot.depot_a = Some(first_step.depot[0]);
            robot.depot_b = Some(first_step.depot[1]);
            robot.depot_c = Some(first_step.depot[2]);
            robot.home_x = Some(home_x);
            robot.home_y = Some(home_y);
            robot.home_size = Some(home_size);
        }

        robot_values.push(robot);
    }

    AnimationRobots {
        robot: robot_values,
    }
}

fn robot_locations(steps: &[RobotAnimationStep], record_depot: bool) -> Vec<AnimationLocation> {
    let mut last_x = 0.0;
    let mut last_y = 0.0;
    let mut last_orientation = 0;
    let mut last_ore_a = 0;
    let mut last_ore_b = 0;
    let mut last_ore_c = 0;
    let mut last_depot_a = 0;
    let mut last_depot_b = 0;
    let mut last_depot_c = 0;
    let mut values = Vec::with_capacity(steps.len());

    for (index, step) in steps.iter().enumerate() {
        let mut location = AnimationLocation::default();

        if index == 0 || step.position.x != last_x {
            location.x = Some(legacy_float(step.position.x));
            last_x = step.position.x;
        }

        if index == 0 || step.position.y != last_y {
            location.y = Some(legacy_float(step.position.y));
            last_y = step.position.y;
        }

        if index == 0 || step.position.orientation != last_orientation {
            location.o = Some(step.position.orientation);
            last_orientation = step.position.orientation;
        }

        if index == 0 || step.ore[0] != last_ore_a {
            location.ore_a = Some(step.ore[0]);
            last_ore_a = step.ore[0];
        }

        if index == 0 || step.ore[1] != last_ore_b {
            location.ore_b = Some(step.ore[1]);
            last_ore_b = step.ore[1];
        }

        if index == 0 || step.ore[2] != last_ore_c {
            location.ore_c = Some(step.ore[2]);
            last_ore_c = step.ore[2];
        }

        if record_depot {
            if index == 0 || step.depot[0] != last_depot_a {
                location.depot_a = Some(step.depot[0]);
                last_depot_a = step.depot[0];
            }
            if index == 0 || step.depot[1] != last_depot_b {
                location.depot_b = Some(step.depot[1]);
                last_depot_b = step.depot[1];
            }
            if index == 0 || step.depot[2] != last_depot_c {
                location.depot_c = Some(step.depot[2]);
                last_depot_c = step.depot[2];
            }
        }

        location.action_index = step.action_index;

        if !step.cpu_steps.is_empty() {
            debug_assert!(
                step.source_line.is_none(),
                "location must not emit both sticky l and cpu"
            );
            location.cpu = Some(
                step.cpu_steps
                    .iter()
                    .cloned()
                    .map(AnimationCpuStep::from)
                    .collect(),
            );
        } else {
            location.source_line = step.source_line;
        }

        if let Some(status) = step.status {
            location.status = Some(status.as_str().to_string());
        }

        let is_empty = location.x.is_none()
            && location.y.is_none()
            && location.o.is_none()
            && location.ore_a.is_none()
            && location.ore_b.is_none()
            && location.ore_c.is_none()
            && location.depot_a.is_none()
            && location.depot_b.is_none()
            && location.depot_c.is_none()
            && location.action_index.is_none()
            && location.source_line.is_none()
            && location.cpu.is_none()
            && location.status.is_none();

        if step.time_fraction < 0.9 || is_empty {
            location.time_fraction = Some(legacy_float(step.time_fraction));
        }

        values.push(location);
    }

    values
}

fn ground_animation(
    ground: &Ground,
    ground_changes: &BTreeMap<(usize, usize), Vec<GroundAnimationStep>>,
) -> AnimationGround {
    let mut positions = Vec::with_capacity(ground_changes.len());

    for ((x, y), changes) in ground_changes {
        positions.push(AnimationGroundPosition {
            x: *x,
            y: *y,
            c: ground_change_array(changes),
        });
    }

    AnimationGround {
        size_x: ground.size_x(),
        size_y: ground.size_y(),
        positions,
    }
}

fn ground_change_array(changes: &[GroundAnimationStep]) -> Vec<AnimationGroundChange> {
    let mut values = Vec::with_capacity(changes.len());

    for change in changes {
        let mut object = AnimationGroundChange::default();

        if change.time > 0 {
            object.time = Some(change.time);
        }
        if change.ore[0] > 0 {
            object.ore_a = Some(change.ore[0]);
        }
        if change.ore[1] > 0 {
            object.ore_b = Some(change.ore[1]);
        }
        if change.ore[2] > 0 {
            object.ore_c = Some(change.ore[2]);
        }

        values.push(object);
    }

    values
}

fn ore_animation(ore_data: &[OreAnimationData]) -> BTreeMap<String, AnimationOreType> {
    let mut object = BTreeMap::new();

    for (index, ore) in ore_data.iter().enumerate() {
        let ore_key = ((b'A' + index as u8) as char).to_string();
        object.insert(
            ore_key,
            AnimationOreType {
                id: ore.ore_id,
                max: ore.max_amount,
            },
        );
    }

    object
}

fn legacy_float(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

/// True when `resultData` looks like pre-JSON executable JavaScript.
pub fn is_legacy_javascript_result_data(result_data: &str) -> bool {
    let trimmed = result_data.trim_start();
    trimmed.starts_with("var myRobots")
        || trimmed.starts_with("var myGround")
        || trimmed.starts_with("var myOreTypes")
}

#[cfg(test)]
mod tests {
    use super::{AnimationPayload, is_legacy_javascript_result_data};

    #[test]
    fn detects_legacy_javascript_payloads() {
        assert!(is_legacy_javascript_result_data(
            "var myRobots = {robot: []};"
        ));
        assert!(!is_legacy_javascript_result_data(
            r#"{"v":2,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#
        ));
    }

    #[test]
    fn parses_minimal_versioned_payload() {
        let payload = AnimationPayload::parse(
            r#"{"v":2,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#,
        )
        .expect("payload should parse");
        assert_eq!(payload.v, 2);
        assert!(payload.robots.robot.is_empty());
        assert_eq!(payload.ground.size_x, 1);
    }
}
