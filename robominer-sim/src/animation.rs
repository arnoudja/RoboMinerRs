use std::collections::BTreeMap;

use crate::MAX_ORE_TYPES;
use crate::ground::Ground;
use crate::position::Position;
use crate::robot::Robot;

pub struct OreAnimationData {
    pub ore_id: i64,
    pub max_amount: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct RobotAnimationStep {
    position: Position,
    ore: [i32; MAX_ORE_TYPES],
    time_fraction: f64,
    /// Optional action index for this cycle (`RobotAction::action_index`, or 0 for scan).
    /// Absent on the initial step and on legacy replays.
    action_index: Option<u8>,
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
    ) {
        self.robot_steps[robot_index].push(RobotAnimationStep {
            position: robot.position(),
            ore: *robot.ore(),
            time_fraction: robot.time_fraction,
            action_index,
        });
    }

    pub(crate) fn into_animation_data(
        self,
        ground: &Ground,
        robots: &[Robot],
        ore_data: &[OreAnimationData],
    ) -> String {
        let mut output = String::new();

        write_robots_animation(&mut output, &self.robot_steps, robots);
        write_ground_animation(&mut output, ground, &self.ground_changes);
        write_ore_animation(&mut output, ore_data);

        output
    }
}

fn write_robots_animation(
    output: &mut String,
    robot_steps: &[Vec<RobotAnimationStep>],
    robots: &[Robot],
) {
    output.push_str("var myRobots = {robot: [");

    for (index, steps) in robot_steps.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        let first_step = steps
            .first()
            .expect("animation should record at least one step per robot");
        let spec = robots[index].spec();

        output.push_str(&format!(
            "{{robotnr:{},x:{},y:{},o:{},A:{},B:{},C:{},size:{},maxore:{},maxturns:{},locations:",
            index,
            format_legacy_float(first_step.position.x),
            format_legacy_float(first_step.position.y),
            first_step.position.orientation,
            first_step.ore[0],
            first_step.ore[1],
            first_step.ore[2],
            format_legacy_float(spec.robot_size),
            spec.max_ore,
            spec.max_turns
        ));
        write_robot_step_array(output, steps);
        output.push_str("}\n");
    }

    output.push_str("]};\n");
}

fn write_robot_step_array(output: &mut String, steps: &[RobotAnimationStep]) {
    output.push('[');

    let mut last_x = 0.0;
    let mut last_y = 0.0;
    let mut last_orientation = 0;
    let mut last_ore_a = 0;
    let mut last_ore_b = 0;
    let mut last_ore_c = 0;

    for (index, step) in steps.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        let mut values = Vec::new();

        if index == 0 || step.position.x != last_x {
            values.push(format!("x:{}", format_legacy_float(step.position.x)));
            last_x = step.position.x;
        }

        if index == 0 || step.position.y != last_y {
            values.push(format!("y:{}", format_legacy_float(step.position.y)));
            last_y = step.position.y;
        }

        if index == 0 || step.position.orientation != last_orientation {
            values.push(format!("o:{}", step.position.orientation));
            last_orientation = step.position.orientation;
        }

        if index == 0 || step.ore[0] != last_ore_a {
            values.push(format!("A:{}", step.ore[0]));
            last_ore_a = step.ore[0];
        }

        if index == 0 || step.ore[1] != last_ore_b {
            values.push(format!("B:{}", step.ore[1]));
            last_ore_b = step.ore[1];
        }

        if index == 0 || step.ore[2] != last_ore_c {
            values.push(format!("C:{}", step.ore[2]));
            last_ore_c = step.ore[2];
        }

        // Always emit when present so Wait cycles stay distinguishable after delta compression.
        if let Some(action_index) = step.action_index {
            values.push(format!("a:{action_index}"));
        }

        if step.time_fraction < 0.9 || values.is_empty() {
            values.push(format!("t:{}", format_legacy_float(step.time_fraction)));
        }

        output.push('{');
        output.push_str(&values.join(","));
        output.push('}');
    }

    output.push(']');
}

fn write_ground_animation(
    output: &mut String,
    ground: &Ground,
    ground_changes: &BTreeMap<(usize, usize), Vec<GroundAnimationStep>>,
) {
    output.push_str(&format!(
        "var myGround = {{sizeX:{},sizeY:{},positions:[",
        ground.size_x(),
        ground.size_y()
    ));

    for (index, ((x, y), changes)) in ground_changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        output.push_str(&format!("{{x:{x},y:{y},c:"));
        write_ground_change_array(output, changes);
        output.push_str("}\n");
    }

    output.push_str("]};\n");
}

fn write_ground_change_array(output: &mut String, changes: &[GroundAnimationStep]) {
    output.push('[');

    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        let mut values = Vec::new();

        if change.time > 0 {
            values.push(format!("t:{}", change.time));
        }
        if change.ore[0] > 0 {
            values.push(format!("A:{}", change.ore[0]));
        }
        if change.ore[1] > 0 {
            values.push(format!("B:{}", change.ore[1]));
        }
        if change.ore[2] > 0 {
            values.push(format!("C:{}", change.ore[2]));
        }

        output.push('{');
        output.push_str(&values.join(","));
        output.push('}');
    }

    output.push(']');
}

fn write_ore_animation(output: &mut String, ore_data: &[OreAnimationData]) {
    output.push_str("var myOreTypes = {");

    for (index, ore) in ore_data.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        let ore_key = (b'A' + index as u8) as char;
        output.push_str(&format!(
            "{ore_key}:{{id:{},max:{}}}",
            ore.ore_id, ore.max_amount
        ));
    }

    output.push_str("};\n");
}

fn format_legacy_float(value: f64) -> String {
    format!("{value:.1}")
}
