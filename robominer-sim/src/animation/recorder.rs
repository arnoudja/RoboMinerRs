use std::collections::BTreeMap;

use crate::MAX_ORE_TYPES;
use crate::animation_payload::AnimationPayload;
use crate::ground::Ground;
use crate::robot::Robot;

use super::serialize::{ground_animation, ore_animation, robots_animation};
use super::types::{
    ANIMATION_PAYLOAD_VERSION, GroundAnimationStep, OreAnimationData, RecordedCpuStep,
    RobotAnimationStep, RobotCycleStatus,
};

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
        let mut payload = AnimationPayload {
            v: ANIMATION_PAYLOAD_VERSION,
            robots: robots_animation(&self.robot_steps, robots, ground.size_x(), ground.size_y()),
            ground: ground_animation(ground, &self.ground_changes),
            ore_types: ore_animation(ore_data),
        };
        // Omit unchanged cpu[].vs up front so long Etaxy-class rallies stay under the
        // persist budget without dropping all locals via strip_cpu_locals.
        crate::animation_payload::sparsify_cpu_locals(&mut payload);
        payload
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
