use super::predecessor_step_met;

fn successor_predecessors_met(predecessors: &[(i64, i32)], claimed_steps: &[(i64, i32)]) -> bool {
    predecessors.iter().all(|(predecessor_id, required_step)| {
        let steps_claimed = claimed_steps
            .iter()
            .find(|(id, _)| id == predecessor_id)
            .map(|(_, steps)| *steps)
            .unwrap_or(0);
        predecessor_step_met(steps_claimed, *required_step)
    })
}

#[test]
fn predecessor_step_met_requires_minimum_claimed_steps() {
    assert!(predecessor_step_met(2, 2));
    assert!(predecessor_step_met(3, 2));
    assert!(!predecessor_step_met(1, 2));
    assert!(!predecessor_step_met(0, 1));
}

#[test]
fn successor_predecessors_met_requires_all_predecessors() {
    let predecessors = [(10, 1), (20, 2)];
    assert!(successor_predecessors_met(
        &predecessors,
        &[(10, 1), (20, 2)],
    ));
    assert!(successor_predecessors_met(
        &predecessors,
        &[(10, 2), (20, 3)],
    ));
    assert!(!successor_predecessors_met(
        &predecessors,
        &[(10, 1), (20, 1)],
    ));
    assert!(!successor_predecessors_met(&predecessors, &[(10, 1)]));
}
