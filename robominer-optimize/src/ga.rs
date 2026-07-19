use crate::fitness::{FitnessContext, evaluate_genome};
use crate::genome::Genome;
use rand::Rng;
use robominer_program::ExecutableProgram;
use std::io::{self, Write};

pub struct GaConfig {
    pub population: usize,
    pub generations: usize,
    pub elite: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub tournament_size: usize,
}

#[derive(Debug, Clone)]
pub struct RankedIndividual {
    pub genome: Genome,
    pub fitness: crate::fitness::FitnessResult,
}

pub fn run_ga(
    config: &GaConfig,
    fitness_ctx: &FitnessContext<'_>,
    initial_programs: &[ExecutableProgram],
    fix_program: bool,
    fixed_parts: Option<[i64; 7]>,
    rng: &mut impl Rng,
) -> Vec<RankedIndividual> {
    if fix_program && let Some(part_ids) = fixed_parts {
        let program = initial_programs
            .first()
            .expect("fix_program requires an injected program")
            .clone();
        let genome = Genome::with_parts(part_ids, program);
        let individual = RankedIndividual {
            fitness: evaluate_genome(&genome, fitness_ctx),
            genome,
        };
        return vec![individual];
    }

    let population_size = config.population.max(2);
    let mut population = build_initial_population(
        fitness_ctx.catalog,
        population_size,
        initial_programs,
        fix_program,
        fixed_parts,
        rng,
    );

    let mut best_ever: Vec<RankedIndividual> = Vec::new();

    for generation in 0..config.generations {
        let mut ranked: Vec<RankedIndividual> = population
            .iter()
            .map(|genome| RankedIndividual {
                fitness: evaluate_genome(genome, fitness_ctx),
                genome: genome.clone(),
            })
            .collect();
        ranked.sort_by(|a, b| {
            b.fitness
                .fitness
                .partial_cmp(&a.fitness.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        merge_best(&mut best_ever, &ranked);

        let best = ranked[0].fitness.fitness;
        let _ = writeln!(
            io::stderr(),
            "generation {generation}: best_fitness={best:.4} pop={}",
            ranked.len()
        );

        let elite_count = config.elite.min(ranked.len());
        let mut next = ranked
            .iter()
            .take(elite_count)
            .map(|individual| individual.genome.clone())
            .collect::<Vec<_>>();

        while next.len() < population_size {
            let parent_a = tournament_select(&ranked, config.tournament_size, rng);
            let parent_b = tournament_select(&ranked, config.tournament_size, rng);
            let (mut child_a, mut child_b) = if rng.r#gen::<f64>() < config.crossover_rate {
                crossover_children(
                    &parent_a.genome,
                    &parent_b.genome,
                    fix_program,
                    fixed_parts,
                    rng,
                )
            } else {
                (parent_a.genome.clone(), parent_b.genome.clone())
            };
            if rng.r#gen::<f64>() < config.mutation_rate {
                child_a = mutate_child(child_a, fitness_ctx.catalog, fix_program, fixed_parts, rng);
            }
            if rng.r#gen::<f64>() < config.mutation_rate {
                child_b = mutate_child(child_b, fitness_ctx.catalog, fix_program, fixed_parts, rng);
            }
            next.push(child_a);
            if next.len() < population_size {
                next.push(child_b);
            }
        }

        population = next;
    }

    // Final evaluation
    let mut ranked: Vec<RankedIndividual> = population
        .iter()
        .map(|genome| RankedIndividual {
            fitness: evaluate_genome(genome, fitness_ctx),
            genome: genome.clone(),
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.fitness
            .fitness
            .partial_cmp(&a.fitness.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    merge_best(&mut best_ever, &ranked);
    best_ever.sort_by(|a, b| {
        b.fitness
            .fitness
            .partial_cmp(&a.fitness.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    best_ever
}

fn build_initial_population(
    catalog: &crate::catalog::PartCatalog,
    population_size: usize,
    initial_programs: &[ExecutableProgram],
    fix_program: bool,
    fixed_parts: Option<[i64; 7]>,
    rng: &mut impl Rng,
) -> Vec<Genome> {
    if fix_program {
        let program = initial_programs
            .first()
            .expect("fix_program requires an injected program");
        return (0..population_size)
            .map(|_| match fixed_parts {
                Some(part_ids) => Genome::with_parts(part_ids, program.clone()),
                None => Genome::with_program(catalog, program.clone(), rng),
            })
            .collect();
    }

    if let Some(part_ids) = fixed_parts {
        let mut population = Vec::with_capacity(population_size);
        for program in initial_programs.iter().take(population_size) {
            population.push(Genome::with_parts(part_ids, program.clone()));
        }
        while population.len() < population_size {
            let program = Genome::random(catalog, rng).program;
            population.push(Genome::with_parts(part_ids, program));
        }
        return population;
    }

    initial_population(catalog, population_size, initial_programs, rng)
}

fn crossover_children(
    parent_a: &Genome,
    parent_b: &Genome,
    fix_program: bool,
    fixed_parts: Option<[i64; 7]>,
    rng: &mut impl Rng,
) -> (Genome, Genome) {
    if fix_program {
        parent_a.crossover_parts(parent_b, rng)
    } else if fixed_parts.is_some() {
        parent_a.crossover_programs_only(parent_b, rng)
    } else {
        parent_a.crossover(parent_b, rng)
    }
}

fn mutate_child(
    child: Genome,
    catalog: &crate::catalog::PartCatalog,
    fix_program: bool,
    fixed_parts: Option<[i64; 7]>,
    rng: &mut impl Rng,
) -> Genome {
    if fix_program {
        child.mutate_parts(catalog, rng)
    } else if fixed_parts.is_some() {
        child.mutate_program_only(rng)
    } else {
        child.mutate(catalog, rng)
    }
}

pub fn initial_population(
    catalog: &crate::catalog::PartCatalog,
    population_size: usize,
    initial_programs: &[ExecutableProgram],
    rng: &mut impl Rng,
) -> Vec<Genome> {
    let mut population = Vec::with_capacity(population_size);
    for program in initial_programs.iter().take(population_size) {
        population.push(Genome::with_program(catalog, program.clone(), rng));
    }
    while population.len() < population_size {
        population.push(Genome::random(catalog, rng));
    }
    population
}

fn tournament_select<'a>(
    ranked: &'a [RankedIndividual],
    tournament_size: usize,
    rng: &mut impl Rng,
) -> &'a RankedIndividual {
    let size = tournament_size.max(1).min(ranked.len());
    let mut best = &ranked[rng.gen_range(0..ranked.len())];
    for _ in 1..size {
        let candidate = &ranked[rng.gen_range(0..ranked.len())];
        if candidate.fitness.fitness > best.fitness.fitness {
            best = candidate;
        }
    }
    best
}

fn merge_best(best_ever: &mut Vec<RankedIndividual>, ranked: &[RankedIndividual]) {
    for individual in ranked.iter().take(5) {
        if !individual.fitness.fitness.is_finite() {
            continue;
        }
        let duplicate = best_ever.iter().any(|existing| {
            existing.genome.part_ids == individual.genome.part_ids
                && existing.fitness.source_code == individual.fitness.source_code
        });
        if !duplicate {
            best_ever.push(individual.clone());
        }
    }
    best_ever.sort_by(|a, b| {
        b.fitness
            .fitness
            .partial_cmp(&a.fitness.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    best_ever.truncate(20);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::PartCatalog;
    use robominer_db::RobotPartRecord;
    use robominer_program::compile_executable_source;

    fn sample_part(id: i64, type_id: i64) -> RobotPartRecord {
        RobotPartRecord {
            id,
            type_id,
            tier_id: Some(1),
            part_name: format!("part-{id}"),
            ore_price_id: 1,
            ore_capacity: 2,
            mining_capacity: 2,
            battery_capacity: 20,
            memory_capacity: 50,
            cpu_capacity: 5,
            forward_capacity: 6,
            backward_capacity: 3,
            rotate_capacity: 2,
            recharge_time: 1,
            scan_time: 1,
            scan_distance: 1,
            weight: 2,
            volume: 8,
            power_usage: 1,
        }
    }

    #[test]
    fn initial_population_prefers_seeded_programs() {
        let parts = (1..=7)
            .map(|type_id| sample_part(type_id, type_id))
            .collect();
        let catalog = PartCatalog::from_parts(parts, 9);
        let seeded = compile_executable_source("mine(); dump();").expect("compile");
        let mut rng = rand::thread_rng();
        let population = initial_population(&catalog, 4, &[seeded.clone()], &mut rng);
        assert_eq!(population.len(), 4);
        assert_eq!(population[0].program.actions(), seeded.actions());
    }

    #[test]
    fn fixed_program_operators_preserve_source() {
        let parts = (1..=7)
            .flat_map(|type_id| {
                [
                    sample_part(type_id * 10, type_id),
                    sample_part(type_id * 10 + 1, type_id),
                ]
            })
            .collect();
        let catalog = PartCatalog::from_parts(parts, 9);
        let program = compile_executable_source("mine();").expect("compile");
        let mut rng = rand::thread_rng();
        let left = Genome::with_program(&catalog, program.clone(), &mut rng);
        let right = Genome::with_program(&catalog, program.clone(), &mut rng);
        let (child_a, child_b) = left.crossover_parts(&right, &mut rng);
        assert_eq!(child_a.program.actions(), program.actions());
        assert_eq!(child_b.program.actions(), program.actions());
        let mutated = child_a.mutate_parts(&catalog, &mut rng);
        assert_eq!(mutated.program.actions(), program.actions());

        let fitness_ctx = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 0,
            seeds: 1,
        };
        let config = GaConfig {
            population: 6,
            generations: 2,
            elite: 1,
            mutation_rate: 1.0,
            crossover_rate: 1.0,
            tournament_size: 2,
        };
        // Empty areas => -inf fitness, so hall-of-fame stays empty; evolution must still run.
        let ranked = run_ga(&config, &fitness_ctx, &[program], true, None, &mut rng);
        assert!(ranked.is_empty());
    }

    #[test]
    fn fixed_parts_and_program_evaluates_once() {
        let parts = (1..=7)
            .map(|type_id| sample_part(type_id * 10, type_id))
            .collect();
        let catalog = PartCatalog::from_parts(parts, 9);
        let program = compile_executable_source("mine();").expect("compile");
        let part_ids = [10, 20, 30, 40, 50, 60, 70];
        let fitness_ctx = FitnessContext {
            areas: &[],
            catalog: &catalog,
            depot_capacity: 0,
            seeds: 1,
        };
        let config = GaConfig {
            population: 6,
            generations: 5,
            elite: 1,
            mutation_rate: 1.0,
            crossover_rate: 1.0,
            tournament_size: 2,
        };
        let mut rng = rand::thread_rng();
        let ranked = run_ga(
            &config,
            &fitness_ctx,
            &[program],
            true,
            Some(part_ids),
            &mut rng,
        );
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].genome.part_ids, part_ids);
        assert_eq!(ranked[0].fitness.source_code, "mine();");
    }
}
