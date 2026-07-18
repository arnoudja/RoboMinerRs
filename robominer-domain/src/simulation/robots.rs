use robominer_sim::ScriptedRobot;

use crate::error::DomainError;
use crate::loadout::RobotLoadout;

pub(crate) fn scripted_robot_from_loadout(
    loadout: &RobotLoadout,
    robot_id_override: Option<i32>,
) -> Result<ScriptedRobot, DomainError> {
    scripted_robot_from_loadout_source(loadout, robot_id_override, &loadout.robot.source_code)
}

pub(crate) fn scripted_robot_from_loadout_source(
    loadout: &RobotLoadout,
    robot_id_override: Option<i32>,
    source_code: &str,
) -> Result<ScriptedRobot, DomainError> {
    let mut spec = loadout.simulator_spec()?;

    if let Some(robot_id) = robot_id_override {
        spec.robot_id = robot_id;
    }

    let program = robominer_program::compile_executable_source(source_code).map_err(|source| {
        DomainError::ProgramCompile {
            robot_id: loadout.robot.id,
            source,
        }
    })?;

    Ok(ScriptedRobot::from_executable_program(spec, &program)
        .with_depot_capacity(loadout.depot_capacity))
}
