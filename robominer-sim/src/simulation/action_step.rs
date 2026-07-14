use crate::physics::{
    check_wall_collision, process_dump, process_move, process_requested_move,
    process_requested_rotation, ActionResult,
};
use crate::robot::RobotAction;

use super::Simulation;

impl Simulation {
    pub(super) fn process_robot_action(
        &mut self,
        robot_index: usize,
        action: RobotAction,
    ) -> ActionResult {
        let preserve_action_result = self.should_preserve_program_action_result(robot_index);
        let robot = &mut self.robots[robot_index];
        robot.actions_done[action.action_index()] += 1;
        if !preserve_action_result {
            self.action_results[robot_index] = None;
        }

        let mut action_result = ActionResult::None;

        match action {
            RobotAction::Wait => {}
            RobotAction::Forward => {
                process_move(robot, robot.spec.forward_speed, robot.time_fraction);
                action_result = ActionResult::Move { direction: 1.0 };
            }
            RobotAction::Backward => {
                process_move(robot, -robot.spec.backward_speed, robot.time_fraction);
                action_result = ActionResult::Move { direction: -1.0 };
            }
            RobotAction::RotateRight => {
                robot.target_rotation = robot.spec.rotate_speed;
                action_result = ActionResult::Value(robot.spec.rotate_speed as f64);
            }
            RobotAction::RotateLeft => {
                robot.target_rotation = -robot.spec.rotate_speed;
                action_result = ActionResult::Value(-(robot.spec.rotate_speed as f64));
            }
            RobotAction::Move(distance) => {
                process_requested_move(robot, distance);
                check_wall_collision(robot);
                return ActionResult::Move {
                    direction: distance.signum(),
                };
            }
            RobotAction::Rotate(rotation) => {
                process_requested_rotation(robot, rotation);
                action_result =
                    ActionResult::Value(robot.target_rotation as f64 * robot.time_fraction);
            }
            RobotAction::Mine => {
                let ground_unit = self.ground.at_position(robot.center_position());
                robot.set_target_mining(ground_unit);
                action_result = ActionResult::Mine;
            }
            RobotAction::DumpAll => {
                let (dumped, change) = process_dump(&mut self.ground, robot, None, self.time);
                if let (Some(animation), Some(change)) = (&mut self.animation, change) {
                    animation.record_ground_change(change.x, change.y, change.time, change.ore);
                }
                action_result = ActionResult::Value(dumped as f64);
            }
            RobotAction::DumpOre(ore_type) => {
                let (dumped, change) =
                    process_dump(&mut self.ground, robot, Some(ore_type), self.time);
                if let (Some(animation), Some(change)) = (&mut self.animation, change) {
                    animation.record_ground_change(change.x, change.y, change.time, change.ore);
                }
                action_result = ActionResult::Value(dumped as f64);
            }
        }

        check_wall_collision(robot);
        action_result
    }
}
