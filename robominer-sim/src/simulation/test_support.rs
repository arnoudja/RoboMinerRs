use crate::action_mapping::PendingExpressionAction;
use crate::physics::ActionResult;

use super::Simulation;

#[cfg(test)]
impl Simulation {
    pub(crate) fn prepare_test_run(&mut self) {
        let max_robot_turns = self
            .robots
            .iter()
            .map(|robot| robot.spec.max_turns)
            .max()
            .unwrap_or(0);
        self.total_moves = self.total_moves.min(max_robot_turns);
        self.init_robot_positions();
        self.time = 0;
        self.process_step();
    }

    pub(crate) fn advance_test_turn(&mut self) {
        self.time += 1;
        self.process_step();
    }

    pub(crate) fn pending_expression_action(
        &self,
        robot_index: usize,
    ) -> Option<PendingExpressionAction> {
        self.pending_expression_actions[robot_index]
    }

    pub(crate) fn test_action_result(&self, robot_index: usize) -> Option<f64> {
        self.action_results[robot_index]
    }

    pub(crate) fn test_action_result_expected(&self, robot_index: usize) -> bool {
        self.action_result_expected[robot_index]
    }

    pub(crate) fn test_record_action_result(&mut self, robot_index: usize, result: ActionResult) {
        self.record_action_result(robot_index, result);
    }

    pub(crate) fn test_set_pending_expression(
        &mut self,
        robot_index: usize,
        pending: Option<PendingExpressionAction>,
    ) {
        self.pending_expression_actions[robot_index] = pending;
    }
}
