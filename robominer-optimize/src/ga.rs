use crate::fitness::{FitnessContext, FitnessResult, evaluate_genome};
use crate::genome::Genome;
use rand::Rng;
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
    pub fitness: FitnessResult,
}

pub fn run_ga(
    config: &GaConfig,
    fitness_ctx: &FitnessContext<'_>,
    rng: &mut impl Rng,
) -> Vec<RankedIndividual> {
    let population_size = config.population.max(2);
    let mut population: Vec<Genome> = (0..population_size)
        .map(|_| Genome::random(fitness_ctx.catalog, rng))
        .collect();

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
                parent_a.genome.crossover(&parent_b.genome, rng)
            } else {
                (parent_a.genome.clone(), parent_b.genome.clone())
            };
            if rng.r#gen::<f64>() < config.mutation_rate {
                child_a = child_a.mutate(fitness_ctx.catalog, rng);
            }
            if rng.r#gen::<f64>() < config.mutation_rate {
                child_b = child_b.mutate(fitness_ctx.catalog, rng);
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
