use crate::catalog::PartCatalog;
use crate::genome::Genome;
use robominer_db::{
    MiningQueueRecord, MiningRallyQueueRecord, RequestedRobotParts, RobotParameters, RobotRecord,
    robot_parameters_for_parts,
};
use robominer_domain::{
    MiningAreaLoadout, RallyLoadout, RallyQueueEntry, RobotLoadout, RobotLoadoutParts,
    run_rally_loadout_with_seed,
};
use robominer_program::{compile_source, unparse_program};
use robominer_sim::MAX_ORE_TYPES;

pub struct FitnessContext<'a> {
    pub areas: &'a [MiningAreaLoadout],
    pub catalog: &'a PartCatalog,
    pub depot_capacity: i32,
    /// How many rally seeds to average per area evaluation.
    pub seed_count: u64,
    /// When true, always use seeds `0..seed_count`. When false, use
    /// `generation * seed_count .. (generation + 1) * seed_count`.
    pub fixed_seeds: bool,
}

/// Rally seeds used for one generation's fitness evaluations.
pub fn rally_seeds_for_generation(
    ctx: &FitnessContext<'_>,
    generation: u64,
) -> std::ops::Range<u64> {
    let count = ctx.seed_count.max(1);
    let base = if ctx.fixed_seeds {
        0
    } else {
        generation.saturating_mul(count)
    };
    base..base + count
}

#[derive(Debug, Clone)]
pub struct FitnessResult {
    pub fitness: f64,
    pub per_area: Vec<(i64, f64)>,
    pub parameters: Option<RobotParameters>,
    pub source_code: String,
    pub compiled_size: Option<usize>,
}

pub fn evaluate_genome(
    genome: &Genome,
    ctx: &FitnessContext<'_>,
    generation: u64,
) -> FitnessResult {
    let source_code = unparse_program(&genome.program);
    let Ok(compiled_size) = compile_source(&source_code) else {
        return invalid_result(source_code);
    };

    let Ok(parts) = ctx.catalog.resolve_parts(&genome.part_ids) else {
        return invalid_result(source_code);
    };

    if parts.memory_module.memory_capacity < compiled_size as i32 {
        return invalid_result(source_code);
    }

    let Some(parameters) = robot_parameters_for_parts(&parts) else {
        return invalid_result(source_code);
    };

    // Zero move/rotate speeds produce Inf time fractions and unstable collision math.
    if parameters.forward_speed <= 0.0
        || parameters.backward_speed <= 0.0
        || parameters.rotate_speed <= 0
        || !parameters.forward_speed.is_finite()
        || !parameters.backward_speed.is_finite()
        || !parameters.robot_size.is_finite()
        || parameters.robot_size <= 0.0
    {
        return invalid_result(source_code);
    }

    let robot = robot_record_from_parts(&parts, &parameters, &source_code);
    let mut depot = [0; MAX_ORE_TYPES];
    depot[0] = ctx.depot_capacity;

    let mut per_area = Vec::with_capacity(ctx.areas.len());
    let mut total = 0.0;
    let mut counted = 0usize;
    let seeds = rally_seeds_for_generation(ctx, generation);
    let seed_count = seeds.end - seeds.start;

    for area in ctx.areas {
        let mut area_total = 0.0;
        for seed in seeds.clone() {
            let loadout = build_rally_loadout(area, &robot, depot);
            let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_rally_loadout_with_seed(&loadout, seed)
            })) {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(_)) | Err(_) => {
                    area_total = f64::NEG_INFINITY;
                    break;
                }
            };
            let score = outcome
                .participants
                .iter()
                .find(|participant| !participant.is_ai)
                .map(|participant| taxed_score(participant.score, area.area.tax_rate))
                .unwrap_or(f64::NEG_INFINITY);
            area_total += score;
        }
        if !area_total.is_finite() {
            return FitnessResult {
                fitness: f64::NEG_INFINITY,
                per_area,
                parameters: Some(parameters),
                source_code,
                compiled_size: Some(compiled_size),
            };
        }
        let area_avg = area_total / seed_count as f64;
        per_area.push((area.area.id, area_avg));
        total += area_avg;
        counted += 1;
    }

    let fitness = if counted == 0 {
        f64::NEG_INFINITY
    } else {
        total / counted as f64
    };

    FitnessResult {
        fitness,
        per_area,
        parameters: Some(parameters),
        source_code,
        compiled_size: Some(compiled_size),
    }
}

fn invalid_result(source_code: String) -> FitnessResult {
    FitnessResult {
        fitness: f64::NEG_INFINITY,
        per_area: Vec::new(),
        parameters: None,
        source_code,
        compiled_size: None,
    }
}

fn taxed_score(raw_score: f64, tax_rate: i32) -> f64 {
    raw_score * (100.0 - f64::from(tax_rate)) / 100.0
}

fn robot_record_from_parts(
    parts: &RequestedRobotParts,
    parameters: &RobotParameters,
    source_code: &str,
) -> RobotRecord {
    RobotRecord {
        id: 11,
        user_id: 3,
        robot_name: "optimizer".to_string(),
        source_code: source_code.to_string(),
        program_source_id: Some(1),
        ore_container_id: Some(parts.ore_container.id),
        mining_unit_id: Some(parts.mining_unit.id),
        battery_id: Some(parts.battery.id),
        memory_module_id: Some(parts.memory_module.id),
        cpu_id: Some(parts.cpu.id),
        engine_id: Some(parts.engine.id),
        ore_scanner_id: Some(parts.ore_scanner.id),
        recharge_time: parameters.recharge_time,
        max_ore: parameters.max_ore,
        mining_speed: parameters.mining_speed,
        max_turns: parameters.max_turns,
        memory_size: parameters.memory_size,
        cpu_speed: parameters.cpu_speed,
        forward_speed: parameters.forward_speed,
        backward_speed: parameters.backward_speed,
        rotate_speed: parameters.rotate_speed,
        robot_size: parameters.robot_size,
        scan_time: parameters.scan_time,
        scan_distance: parameters.scan_distance,
        total_mining_runs: 0,
    }
}

fn build_rally_loadout(
    area: &MiningAreaLoadout,
    robot: &RobotRecord,
    depot: [i32; MAX_ORE_TYPES],
) -> RallyLoadout {
    let queue = RallyQueueEntry::new(
        MiningRallyQueueRecord {
            queue: MiningQueueRecord {
                id: 100,
                mining_area_id: area.area.id,
                robot_id: robot.id,
                rally_result_id: None,
                player_number: Some(1),
                score: None,
                claimed: false,
            },
            user_id: robot.user_id,
            seconds_left: 0,
        },
        RobotLoadout::new(robot.clone(), RobotLoadoutParts::empty()).with_depot_capacity(depot),
    );
    RallyLoadout::new(area.clone(), vec![queue])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::PartCatalog;
    use robominer_db::RobotPartRecord;
    use robominer_domain::{MiningAreaLoadout, RobotLoadout, RobotLoadoutParts};
    use robominer_program::compile_executable_source;
    use robominer_test_support::{
        ore_supply_record, unit_test_mining_area_record, unit_test_robot_record,
    };

    fn sample_part(id: i64, type_id: i64, memory: i32) -> RobotPartRecord {
        part_with_caps(id, type_id, memory, 2, 8, 1, 6, 3, 2)
    }

    fn part_with_caps(
        id: i64,
        type_id: i64,
        memory: i32,
        weight: i32,
        volume: i32,
        power_usage: i32,
        forward: i32,
        backward: i32,
        rotate: i32,
    ) -> RobotPartRecord {
        RobotPartRecord {
            id,
            type_id,
            tier_id: Some(1),
            part_name: format!("part-{id}"),
            ore_price_id: 1,
            ore_capacity: 2,
            mining_capacity: 2,
            battery_capacity: 20,
            memory_capacity: memory,
            cpu_capacity: 5,
            forward_capacity: forward,
            backward_capacity: backward,
            rotate_capacity: rotate,
            recharge_time: 1,
            scan_time: 1,
            scan_distance: 1,
            weight,
            volume,
            power_usage,
        }
    }

    fn complete_catalog(memory: i32) -> PartCatalog {
        PartCatalog::from_parts(
            (1..=7)
                .map(|type_id| sample_part(type_id * 10, type_id, memory))
                .collect(),
            9999,
        )
    }

    fn tiny_area() -> MiningAreaLoadout {
        let mut area = unit_test_mining_area_record(1001);
        area.max_moves = 3;
        MiningAreaLoadout::new(
            area,
            vec![ore_supply_record(1, 1001, 1, 10, 2)],
            RobotLoadout::new(
                unit_test_robot_record(1, "rotate(90);"),
                RobotLoadoutParts::empty(),
            ),
        )
    }

    #[test]
    fn fitness_rejects_program_larger_than_memory() {
        let catalog = complete_catalog(1);
        let program = compile_executable_source("move(1); mine();").expect("compile");
        let genome = Genome {
            part_ids: [10, 20, 30, 40, 50, 60, 70],
            program,
        };
        let ctx = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 40,
            seed_count: 1,
            fixed_seeds: true,
        };
        let result = evaluate_genome(&genome, &ctx, 0);
        assert!(!result.fitness.is_finite());
        assert!(result.parameters.is_none());
    }

    #[test]
    fn fitness_rejects_unknown_parts_and_zero_power() {
        let catalog = complete_catalog(50);
        let program = compile_executable_source("mine();").expect("compile");

        let unknown = Genome {
            part_ids: [999, 20, 30, 40, 50, 60, 70],
            program: program.clone(),
        };
        let ctx = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 0,
            seed_count: 1,
            fixed_seeds: true,
        };
        assert!(!evaluate_genome(&unknown, &ctx, 0).fitness.is_finite());

        let zero_power = PartCatalog::from_parts(
            (1..=7)
                .map(|type_id| part_with_caps(type_id * 10, type_id, 50, 2, 8, 0, 6, 3, 2))
                .collect(),
            9,
        );
        let genome = Genome {
            part_ids: [10, 20, 30, 40, 50, 60, 70],
            program,
        };
        let ctx = FitnessContext {
            areas: &[],
            catalog: &zero_power,
            depot_capacity: 0,
            seed_count: 1,
            fixed_seeds: true,
        };
        assert!(!evaluate_genome(&genome, &ctx, 0).fitness.is_finite());
    }

    #[test]
    fn fitness_rejects_non_positive_move_speeds() {
        let catalog = PartCatalog::from_parts(
            (1..=7)
                .map(|type_id| part_with_caps(type_id * 10, type_id, 50, 2, 8, 1, 0, 0, 0))
                .collect(),
            9,
        );
        let program = compile_executable_source("mine();").expect("compile");
        let genome = Genome {
            part_ids: [10, 20, 30, 40, 50, 60, 70],
            program,
        };
        let ctx = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 0,
            seed_count: 1,
            fixed_seeds: true,
        };
        assert!(!evaluate_genome(&genome, &ctx, 0).fitness.is_finite());
    }

    #[test]
    fn fitness_empty_areas_yields_negative_infinity_with_parameters() {
        let catalog = complete_catalog(50);
        let program = compile_executable_source("mine();").expect("compile");
        let genome = Genome {
            part_ids: [10, 20, 30, 40, 50, 60, 70],
            program,
        };
        let ctx = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 40,
            seed_count: 2,
            fixed_seeds: true,
        };
        let result = evaluate_genome(&genome, &ctx, 0);
        assert!(!result.fitness.is_finite());
        assert!(result.parameters.is_some());
        assert!(result.compiled_size.is_some());
        assert!(result.per_area.is_empty());
    }

    #[test]
    fn fitness_scores_tiny_rally_area() {
        let catalog = complete_catalog(50);
        let program = compile_executable_source("mine();").expect("compile");
        let genome = Genome {
            part_ids: [10, 20, 30, 40, 50, 60, 70],
            program,
        };
        let area = tiny_area();
        let ctx = FitnessContext {
            areas: std::slice::from_ref(&area),
            catalog: &catalog,
            depot_capacity: 10,
            seed_count: 2,
            fixed_seeds: true,
        };
        let result = evaluate_genome(&genome, &ctx, 0);
        assert!(result.fitness.is_finite(), "fitness={}", result.fitness);
        assert_eq!(result.per_area.len(), 1);
        assert_eq!(result.per_area[0].0, 1001);
        assert!(result.parameters.is_some());
        assert_eq!(result.source_code, "mine();");
    }

    #[test]
    fn taxed_score_applies_tax_rate() {
        assert!((taxed_score(100.0, 25) - 75.0).abs() < f64::EPSILON);
        assert!((taxed_score(40.0, 0) - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rotating_seeds_advance_each_generation() {
        let catalog = complete_catalog(50);
        let rotating = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 0,
            seed_count: 5,
            fixed_seeds: false,
        };
        assert_eq!(rally_seeds_for_generation(&rotating, 0), 0..5);
        assert_eq!(rally_seeds_for_generation(&rotating, 1), 5..10);
        assert_eq!(rally_seeds_for_generation(&rotating, 3), 15..20);

        let fixed = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 0,
            seed_count: 5,
            fixed_seeds: true,
        };
        assert_eq!(rally_seeds_for_generation(&fixed, 0), 0..5);
        assert_eq!(rally_seeds_for_generation(&fixed, 3), 0..5);

        let zero_seeds = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 0,
            seed_count: 0,
            fixed_seeds: false,
        };
        assert_eq!(rally_seeds_for_generation(&zero_seeds, 2), 2..3);
    }
}
