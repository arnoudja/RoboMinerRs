use robominer_program::{ExecutableProgram, ExecutableRunner};

use crate::ground::{GroundUnit, ScanSnapshot, ScanState};
use crate::action_mapping::expand_executable_actions;
use crate::physics::position_at_time;
use crate::score::calculate_score;
use crate::MAX_ORE_TYPES;
use crate::position::Position;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RobotAction {
    Wait,
    Forward,
    Backward,
    RotateRight,
    RotateLeft,
    Move(f64),
    Rotate(f64),
    Mine,
    DumpAll,
    DumpOre(usize),
}

impl RobotAction {
    pub(crate) fn action_index(self) -> usize {
        match self {
            RobotAction::Wait => 1,
            RobotAction::Forward => 2,
            RobotAction::Backward => 3,
            RobotAction::RotateRight => 4,
            RobotAction::RotateLeft => 5,
            RobotAction::Move(distance) if distance > 0.0 => 2,
            RobotAction::Move(distance) if distance < 0.0 => 3,
            RobotAction::Move(_) => 1,
            RobotAction::Rotate(rotation) if rotation > 0.0 => 4,
            RobotAction::Rotate(rotation) if rotation < 0.0 => 5,
            RobotAction::Rotate(_) => 1,
            RobotAction::Mine => 6,
            RobotAction::DumpAll | RobotAction::DumpOre(_) => 7,
        }
    }
}

pub const ROBOT_ACTION_TYPE_SCAN: usize = 0;

#[derive(Clone, Debug)]
pub struct RobotSpec {
    pub robot_id: i32,
    pub max_turns: i32,
    pub max_ore: i32,
    pub mining_speed: i32,
    pub cpu_speed: i32,
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: i32,
    pub robot_size: f64,
    pub scan_time: i32,
    pub scan_distance: i32,
}

impl RobotSpec {
    pub fn test_robot() -> Self {
        Self {
            robot_id: 1,
            max_turns: 10,
            max_ore: 100,
            mining_speed: 5,
            cpu_speed: 72,
            forward_speed: 1.0,
            backward_speed: 1.0,
            rotate_speed: 90,
            robot_size: 1.0,
            scan_time: 6,
            scan_distance: 5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Robot {
    pub(crate) spec: RobotSpec,
    pub(crate) position: Position,
    pub(crate) destination: Position,
    pub(crate) ore: [i32; MAX_ORE_TYPES],
    pub(crate) target_mining: [i32; MAX_ORE_TYPES],
    pub(crate) last_mined: i32,
    pub(crate) time_left: i32,
    pub(crate) time_fraction: f64,
    pub(crate) current_speed: f64,
    pub(crate) target_rotation: i32,
    pub(crate) min_x: f64,
    pub(crate) min_y: f64,
    pub(crate) max_x: f64,
    pub(crate) max_y: f64,
    pub(crate) actions_done: [i32; 8],
    pub(crate) scan_state: ScanState,
    pub(crate) initial_center_x: f64,
    pub(crate) initial_center_y: f64,
    pub(crate) initial_orientation: i32,
}

impl Robot {
    pub(crate) fn new(spec: RobotSpec) -> Self {
        Self {
            spec,
            position: Position::default(),
            destination: Position::default(),
            ore: [0; MAX_ORE_TYPES],
            target_mining: [0; MAX_ORE_TYPES],
            last_mined: 0,
            time_left: 0,
            time_fraction: 0.0,
            current_speed: 0.0,
            target_rotation: 0,
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
            actions_done: [0; 8],
            scan_state: ScanState::Idle,
            initial_center_x: 0.0,
            initial_center_y: 0.0,
            initial_orientation: 0,
        }
    }

    pub(crate) fn scan_snapshot(&self) -> ScanSnapshot {
        match &self.scan_state {
            ScanState::Idle => ScanSnapshot {
                started: false,
                complete: false,
                distance: -1.0,
                ore_type: 0.0,
            },
            ScanState::Scanning { .. } => ScanSnapshot {
                started: true,
                complete: false,
                distance: -1.0,
                ore_type: 0.0,
            },
            ScanState::Complete(result) => ScanSnapshot {
                started: true,
                complete: true,
                distance: result.distance,
                ore_type: result.ore_type,
            },
        }
    }

    pub fn spec(&self) -> &RobotSpec {
        &self.spec
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn ore(&self) -> &[i32; MAX_ORE_TYPES] {
        &self.ore
    }

    pub fn ore_at(&self, ore_type: usize) -> i32 {
        self.ore[ore_type]
    }

    pub fn total_ore(&self) -> i32 {
        self.ore.iter().sum()
    }

    pub fn last_mined(&self) -> i32 {
        self.last_mined
    }

    pub fn time_left(&self) -> i32 {
        self.time_left
    }

    pub fn actions_done(&self) -> &[i32; 8] {
        &self.actions_done
    }

    pub fn calculate_score(&self) -> f64 {
        calculate_score(self.ore)
    }

    pub(crate) fn center_position(&self) -> Position {
        Position::new(
            self.position.x + self.spec.robot_size / 2.0,
            self.position.y + self.spec.robot_size / 2.0,
            self.position.orientation,
        )
    }

    pub(crate) fn effective_center_position(&self) -> Position {
        if self.time_fraction >= 1.0 - f64::EPSILON
            || (self.position.x - self.destination.x).abs() < f64::EPSILON
                && (self.position.y - self.destination.y).abs() < f64::EPSILON
        {
            return self.center_position();
        }

        let interpolated = position_at_time(
            self.position,
            self.destination,
            self.time_fraction,
            self.time_fraction,
        );
        Position::new(
            interpolated.x + self.spec.robot_size / 2.0,
            interpolated.y + self.spec.robot_size / 2.0,
            interpolated.orientation,
        )
    }

    pub(crate) fn prepare_for_action(&mut self, current_step: i32, max_steps: i32) {
        self.time_left = self.spec.max_turns.min(max_steps) - current_step;
        self.destination = self.position;
        self.time_fraction = 1.0;
        self.current_speed = 0.0;
        self.target_rotation = 0;
        self.target_mining = [0; MAX_ORE_TYPES];
    }

    pub(crate) fn apply_rotation(&mut self) {
        let angle = (self.time_fraction * self.target_rotation as f64) as i32;
        self.position.rotate(angle);
    }

    pub(crate) fn set_target_mining(&mut self, ground_unit: &GroundUnit) {
        self.last_mined = 0;

        let mut total_allowed = self
            .spec
            .mining_speed
            .min(self.spec.max_ore - self.total_ore());
        let mut ore_types = ground_unit.ore_type_count();

        for ore_type in 0..MAX_ORE_TYPES {
            if ground_unit.ore_at(ore_type) > 0 {
                let max_allowed = total_allowed.min(((total_allowed - 1) / ore_types) + 1);
                let available = ((ground_unit.ore_at(ore_type) - 1) / 2) + 1;

                self.target_mining[ore_type] = available.min(max_allowed);
                total_allowed -= self.target_mining[ore_type];
                ore_types -= 1;
            }
        }
    }

    pub(crate) fn add_ore(&mut self, ore_type: usize, amount: i32) {
        self.ore[ore_type] += amount;
        self.last_mined += amount;
    }

    pub(crate) fn clear_ore(&mut self, ore_type: usize) {
        self.ore[ore_type] = 0;
    }
}

#[derive(Clone, Debug)]
pub struct ScriptedRobot {
    pub(crate) spec: RobotSpec,
    pub(crate) action_source: ActionSource,
}

#[derive(Clone, Debug)]
pub(crate) enum ActionSource {
    Actions(Vec<RobotAction>),
    RepeatingActions(Vec<RobotAction>),
    Program {
        program: Box<ExecutableProgram>,
        runner: Box<ExecutableRunner>,
    },
}

impl ScriptedRobot {
    pub fn new(spec: RobotSpec, actions: Vec<RobotAction>) -> Self {
        Self {
            spec,
            action_source: ActionSource::Actions(actions),
        }
    }

    pub fn from_executable_program(spec: RobotSpec, program: &ExecutableProgram) -> Self {
        if program.requires_runtime() {
            Self {
                spec,
                action_source: ActionSource::Program {
                    program: Box::new(program.clone()),
                    runner: Box::new(program.runner()),
                },
            }
        } else {
            let actions = expand_executable_actions(&spec, program.actions());

            Self {
                spec,
                action_source: ActionSource::RepeatingActions(actions),
            }
        }
    }
}

