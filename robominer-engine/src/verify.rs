use anyhow::{Context, Result, anyhow, ensure};
use std::fs;
use std::path::PathBuf;

pub(crate) struct SimulateSourceOptions {
    pub(crate) source_file: Option<PathBuf>,
    pub(crate) robot_files: Vec<PathBuf>,
    pub(crate) turns: i32,
    pub(crate) size_x: usize,
    pub(crate) size_y: usize,
    pub(crate) ore_x: usize,
    pub(crate) ore_y: usize,
    pub(crate) ore_type: usize,
    pub(crate) ore_amount: i32,
    pub(crate) mining_speed: i32,
    pub(crate) forward_speed: f64,
    pub(crate) backward_speed: f64,
    pub(crate) rotate_speed: i32,
}

pub(crate) async fn verify(pool: &robominer_db::MySqlPool, program_source_id: i64) -> Result<()> {
    let source = robominer_db::get_program_source(pool, program_source_id)
        .await
        .context("failed to load program source")?
        .ok_or_else(|| anyhow!("program source {program_source_id} not found"))?;

    mark_program_source_verification(pool, program_source_id, &source).await
}

pub(crate) async fn mark_program_source_verification(
    pool: &robominer_db::MySqlPool,
    program_source_id: i64,
    source: &str,
) -> Result<()> {
    let source = source.to_owned();
    let verification = tokio::task::spawn_blocking(move || robominer_program::verify_source(&source))
        .await
        .context("program verification task failed")?;

    if verification.verified {
        robominer_db::set_valid_program_source(pool, program_source_id, verification.compiled_size)
            .await
            .context("failed to mark program source as valid")?;

        println!(
            "Program source {program_source_id} verified, compiled size {}",
            verification.compiled_size
        );
    } else {
        robominer_db::set_invalid_program_source(
            pool,
            program_source_id,
            &verification.error_description,
        )
        .await
        .context("failed to mark program source as invalid")?;

        println!(
            "Program source {program_source_id} invalid: {}",
            verification.error_description
        );
    }

    Ok(())
}

pub(crate) fn verify_source_file(source_file: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(source_file)
        .with_context(|| format!("failed to read source file {}", source_file.display()))?;
    let verification = robominer_program::verify_source(&source);

    if verification.verified {
        println!(
            "{} verified, compiled size {}",
            source_file.display(),
            verification.compiled_size
        );
    } else {
        println!(
            "{} invalid: {}",
            source_file.display(),
            verification.error_description
        );
    }

    Ok(())
}

pub(crate) fn simulate_source_file(options: SimulateSourceOptions) -> Result<()> {
    ensure!(options.turns >= 0, "--turns must be non-negative");
    ensure!(options.size_x > 0, "--size-x must be greater than zero");
    ensure!(options.size_y > 0, "--size-y must be greater than zero");
    ensure!(
        options.ore_x < options.size_x,
        "--ore-x must be inside the map"
    );
    ensure!(
        options.ore_y < options.size_y,
        "--ore-y must be inside the map"
    );
    ensure!(
        options.ore_type < robominer_sim::MAX_ORE_TYPES,
        "--ore-type must be 0..9"
    );
    ensure!(options.ore_amount >= 0, "--ore-amount must be non-negative");
    ensure!(
        options.mining_speed >= 0,
        "--mining-speed must be non-negative"
    );
    ensure!(
        options.forward_speed >= 0.0,
        "--forward-speed must be non-negative"
    );
    ensure!(
        options.backward_speed >= 0.0,
        "--backward-speed must be non-negative"
    );
    ensure!(
        options.rotate_speed >= 0,
        "--rotate-speed must be non-negative"
    );

    let robot_files = simulation_robot_files(&options)?;
    ensure!(
        robot_files.len() <= 4,
        "simulate-source supports at most four robots"
    );

    let mut programs = Vec::new();
    for source_file in &robot_files {
        let source = fs::read_to_string(source_file)
            .with_context(|| format!("failed to read source file {}", source_file.display()))?;
        let program = robominer_program::compile_executable_source(&source).with_context(|| {
            format!(
                "failed to compile executable program {}",
                source_file.display()
            )
        })?;
        programs.push(program);
    }

    let mut ground = robominer_sim::Ground::new(options.size_x, options.size_y);
    if options.ore_amount > 0 {
        ground
            .at_mut(options.ore_x, options.ore_y)
            .add_ore(options.ore_type, options.ore_amount);
    }

    let robots: Vec<_> = programs
        .iter()
        .enumerate()
        .map(|(index, program)| {
            let mut spec = robominer_sim::RobotSpec::test_robot();
            spec.robot_id = (index + 1) as i32;
            spec.max_turns = options.turns;
            spec.mining_speed = options.mining_speed;
            spec.forward_speed = options.forward_speed;
            spec.backward_speed = options.backward_speed;
            spec.rotate_speed = options.rotate_speed;

            robominer_sim::ScriptedRobot::from_executable_program(spec, program)
        })
        .collect();

    let mut simulation = robominer_sim::Simulation::new(ground, options.turns, robots);
    simulation.run();

    println!("Simulation complete");
    println!("turns: {}", simulation.time());
    println!("robots: {}", robot_files.len());

    for (index, source_file) in robot_files.iter().enumerate() {
        let robot = simulation.robot(index);
        let position = robot.position();
        let actions = robot.actions_done();

        println!("robot {} source: {}", index + 1, source_file.display());
        println!(
            "robot {} position: x={:.3} y={:.3} orientation={}",
            index + 1,
            position.x,
            position.y,
            position.orientation
        );
        println!("robot {} ore: {:?}", index + 1, robot.ore());
        println!("robot {} score: {:.3}", index + 1, robot.calculate_score());
        println!(
            "robot {} actions: wait={} forward={} backward={} rotate_right={} rotate_left={} mine={} dump={}",
            index + 1,
            actions[1],
            actions[2],
            actions[3],
            actions[4],
            actions[5],
            actions[6],
            actions[7]
        );
    }

    for first in 0..robot_files.len().saturating_sub(1) {
        for second in (first + 1)..robot_files.len() {
            let distance = simulation
                .robot(first)
                .position()
                .distance(&simulation.robot(second).position());
            println!(
                "distance robot {}-{}: {:.3}",
                first + 1,
                second + 1,
                distance
            );
        }
    }

    Ok(())
}

fn simulation_robot_files(options: &SimulateSourceOptions) -> Result<Vec<PathBuf>> {
    let mut robot_files = options.robot_files.clone();

    if let Some(source_file) = &options.source_file {
        ensure!(
            robot_files.is_empty(),
            "provide either a positional source file or --robot files, not both"
        );
        robot_files.push(source_file.clone());
    }

    ensure!(
        !robot_files.is_empty(),
        "provide a source file or at least one --robot file"
    );

    Ok(robot_files)
}
