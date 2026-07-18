use std::collections::BTreeMap;

use serde_json::{Map, Number, Value, json};

use crate::MAX_ORE_TYPES;
use crate::ground::Ground;
use crate::position::Position;
use crate::robot::Robot;

/// Current on-disk / wire format for rally animation payloads stored in
/// `RallyResult.resultData`. Older executable JavaScript rows (`var myRobots = …`)
/// are no longer played by the web viewer.
pub const ANIMATION_PAYLOAD_VERSION: u32 = 1;

pub struct OreAnimationData {
    pub ore_id: i64,
    pub max_amount: i32,
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
    /// Non-zero motion requested but no chunk could be issued (e.g. zero speed).
    Motion,
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
            Self::Motion => "motion",
            Self::Wait => "wait",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RobotAnimationStep {
    position: Position,
    ore: [i32; MAX_ORE_TYPES],
    time_fraction: f64,
    /// Optional action index for this cycle (`RobotAction::action_index`, or 0 for scan).
    /// Absent on the initial step and on legacy replays.
    action_index: Option<u8>,
    /// Optional 1-based source line of the statement executing this cycle.
    /// Absent on the initial step, scripted action lists, and legacy replays.
    source_line: Option<u16>,
    /// Optional stuck/idle reason for this cycle.
    status: Option<RobotCycleStatus>,
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
    ) {
        self.robot_steps[robot_index].push(RobotAnimationStep {
            position: robot.position(),
            ore: *robot.ore(),
            time_fraction: robot.time_fraction,
            action_index,
            source_line,
            status,
        });
    }

    pub(crate) fn into_animation_data(
        self,
        ground: &Ground,
        robots: &[Robot],
        ore_data: &[OreAnimationData],
    ) -> String {
        let payload = json!({
            "v": ANIMATION_PAYLOAD_VERSION,
            "robots": robots_animation_value(&self.robot_steps, robots),
            "ground": ground_animation_value(ground, &self.ground_changes),
            "oreTypes": ore_animation_value(ore_data),
        });
        // Prevent `</script>` breakout when the JSON is embedded in HTML.
        payload.to_string().replace('<', "\\u003c")
    }
}

fn robots_animation_value(robot_steps: &[Vec<RobotAnimationStep>], robots: &[Robot]) -> Value {
    let mut robot_values = Vec::with_capacity(robot_steps.len());

    for (index, steps) in robot_steps.iter().enumerate() {
        let first_step = steps
            .first()
            .expect("animation should record at least one step per robot");
        let spec = robots[index].spec();

        robot_values.push(json!({
            "robotnr": index,
            "x": legacy_float(first_step.position.x),
            "y": legacy_float(first_step.position.y),
            "o": first_step.position.orientation,
            "A": first_step.ore[0],
            "B": first_step.ore[1],
            "C": first_step.ore[2],
            "size": legacy_float(spec.robot_size),
            "maxore": spec.max_ore,
            "maxturns": spec.max_turns,
            "locations": robot_step_array_value(steps),
        }));
    }

    json!({ "robot": robot_values })
}

fn robot_step_array_value(steps: &[RobotAnimationStep]) -> Value {
    let mut last_x = 0.0;
    let mut last_y = 0.0;
    let mut last_orientation = 0;
    let mut last_ore_a = 0;
    let mut last_ore_b = 0;
    let mut last_ore_c = 0;
    let mut values = Vec::with_capacity(steps.len());

    for (index, step) in steps.iter().enumerate() {
        let mut object = Map::new();

        if index == 0 || step.position.x != last_x {
            object.insert("x".to_string(), json!(legacy_float(step.position.x)));
            last_x = step.position.x;
        }

        if index == 0 || step.position.y != last_y {
            object.insert("y".to_string(), json!(legacy_float(step.position.y)));
            last_y = step.position.y;
        }

        if index == 0 || step.position.orientation != last_orientation {
            object.insert("o".to_string(), json!(step.position.orientation));
            last_orientation = step.position.orientation;
        }

        if index == 0 || step.ore[0] != last_ore_a {
            object.insert("A".to_string(), json!(step.ore[0]));
            last_ore_a = step.ore[0];
        }

        if index == 0 || step.ore[1] != last_ore_b {
            object.insert("B".to_string(), json!(step.ore[1]));
            last_ore_b = step.ore[1];
        }

        if index == 0 || step.ore[2] != last_ore_c {
            object.insert("C".to_string(), json!(step.ore[2]));
            last_ore_c = step.ore[2];
        }

        // Always emit when present so Wait cycles stay distinguishable after delta compression.
        if let Some(action_index) = step.action_index {
            object.insert("a".to_string(), json!(action_index));
        }

        // Always emit when present so the viewer can highlight the active statement.
        if let Some(source_line) = step.source_line {
            object.insert("l".to_string(), json!(source_line));
        }

        // Always emit when present so stuck reasons survive delta compression.
        if let Some(status) = step.status {
            object.insert("s".to_string(), json!(status.as_str()));
        }

        if step.time_fraction < 0.9 || object.is_empty() {
            object.insert("t".to_string(), json!(legacy_float(step.time_fraction)));
        }

        values.push(Value::Object(object));
    }

    Value::Array(values)
}

fn ground_animation_value(
    ground: &Ground,
    ground_changes: &BTreeMap<(usize, usize), Vec<GroundAnimationStep>>,
) -> Value {
    let mut positions = Vec::with_capacity(ground_changes.len());

    for ((x, y), changes) in ground_changes {
        positions.push(json!({
            "x": x,
            "y": y,
            "c": ground_change_array_value(changes),
        }));
    }

    json!({
        "sizeX": ground.size_x(),
        "sizeY": ground.size_y(),
        "positions": positions,
    })
}

fn ground_change_array_value(changes: &[GroundAnimationStep]) -> Value {
    let mut values = Vec::with_capacity(changes.len());

    for change in changes {
        let mut object = Map::new();

        if change.time > 0 {
            object.insert("t".to_string(), json!(change.time));
        }
        if change.ore[0] > 0 {
            object.insert("A".to_string(), json!(change.ore[0]));
        }
        if change.ore[1] > 0 {
            object.insert("B".to_string(), json!(change.ore[1]));
        }
        if change.ore[2] > 0 {
            object.insert("C".to_string(), json!(change.ore[2]));
        }

        values.push(Value::Object(object));
    }

    Value::Array(values)
}

fn ore_animation_value(ore_data: &[OreAnimationData]) -> Value {
    let mut object = Map::new();

    for (index, ore) in ore_data.iter().enumerate() {
        let ore_key = ((b'A' + index as u8) as char).to_string();
        object.insert(
            ore_key,
            json!({
                "id": ore.ore_id,
                "max": ore.max_amount,
            }),
        );
    }

    Value::Object(object)
}

fn legacy_float(value: f64) -> Value {
    let rounded = (value * 10.0).round() / 10.0;
    Number::from_f64(rounded)
        .map(Value::Number)
        .unwrap_or(Value::Null)
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
    use super::is_legacy_javascript_result_data;

    #[test]
    fn detects_legacy_javascript_payloads() {
        assert!(is_legacy_javascript_result_data("var myRobots = {robot: []};"));
        assert!(!is_legacy_javascript_result_data(
            r#"{"v":1,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#
        ));
    }
}
