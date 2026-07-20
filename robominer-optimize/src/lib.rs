mod catalog;
mod cli;
mod fitness;
mod ga;
mod genome;
mod report;

pub use catalog::PartCatalog;
pub use cli::Cli;
pub use fitness::{FitnessContext, FitnessResult, evaluate_genome, rally_seeds_for_generation};
pub use ga::{GaConfig, RankedIndividual, initial_population, run_ga};
pub use genome::Genome;
pub use report::format_top_results;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use rand::SeedableRng;
use rand::rngs::StdRng;
use robominer_domain::load_mining_area_loadout;
use robominer_program::{ExecutableProgram, compile_executable_source};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let pool = connect_database(cli.database_url.clone(), cli.config.clone()).await?;

    let catalog = PartCatalog::load(&pool, cli.max_tier_id)
        .await
        .context("failed to load robot part catalog")?;
    if !catalog.is_complete() {
        return Err(anyhow!(
            "part catalog is incomplete for max-tier-id {}; need at least one part in each of types 1-7",
            cli.max_tier_id
        ));
    }

    let mut areas = Vec::new();
    for area_id in &cli.areas {
        let loadout = load_mining_area_loadout(&pool, *area_id)
            .await
            .with_context(|| format!("failed to load mining area {area_id}"))?
            .ok_or_else(|| anyhow!("mining area {area_id} not found"))?;
        areas.push(loadout);
    }

    let initial_programs = load_initial_programs(&cli.programs, &cli.program_files)?;
    if cli.evaluate_only {
        if initial_programs.len() != 1 {
            return Err(anyhow!(
                "--evaluate-only requires exactly one --program or --program-file"
            ));
        }
    } else if initial_programs.len() > cli.population.max(2) {
        eprintln!(
            "warning: {} initial programs exceed population {}; extras are ignored",
            initial_programs.len(),
            cli.population.max(2)
        );
    }

    let fixed_parts = parse_fixed_parts(&catalog, &cli.parts)?;

    // Default: rotate 5 seeds per generation. Explicit --seeds N keeps 0..N-1 fixed.
    const DEFAULT_SEED_COUNT: u64 = 5;
    let (seed_count, fixed_seeds) = match cli.seeds {
        Some(seeds) => (seeds.max(1), true),
        None => (DEFAULT_SEED_COUNT, false),
    };
    if fixed_seeds {
        eprintln!("using fixed rally seeds 0..{seed_count} every generation");
    } else {
        eprintln!(
            "rotating {seed_count} rally seeds each generation (pass --seeds N to keep 0..N-1 fixed)"
        );
    }

    let mut rng = StdRng::seed_from_u64(cli.seed);
    let fitness_ctx = FitnessContext {
        areas: &areas,
        catalog: &catalog,
        depot_capacity: cli.depot_capacity,
        seed_count,
        fixed_seeds,
    };
    let ga_config = GaConfig {
        population: cli.population,
        generations: cli.generations,
        elite: cli.elite,
        mutation_rate: cli.mutation_rate,
        crossover_rate: cli.crossover_rate,
        tournament_size: cli.tournament_size,
    };

    let ranked = run_ga(
        &ga_config,
        &fitness_ctx,
        &initial_programs,
        cli.evaluate_only,
        fixed_parts,
        &mut rng,
    );
    let report = format_top_results(&ranked, &catalog, cli.depot_capacity, cli.top_n);
    println!("{report}");
    Ok(())
}

fn parse_fixed_parts(catalog: &PartCatalog, parts: &[i64]) -> Result<Option<[i64; 7]>> {
    if parts.is_empty() {
        return Ok(None);
    }
    if parts.len() != 7 {
        return Err(anyhow!(
            "--parts requires exactly 7 ids (got {}); order is container,mining,battery,memory,cpu,engine,scanner",
            parts.len()
        ));
    }
    let mut part_ids = [0_i64; 7];
    part_ids.copy_from_slice(parts);
    catalog
        .resolve_parts(&part_ids)
        .with_context(|| format!("invalid --parts {parts:?}"))?;
    Ok(Some(part_ids))
}

fn load_initial_programs(
    programs: &[String],
    program_files: &[std::path::PathBuf],
) -> Result<Vec<ExecutableProgram>> {
    let mut compiled = Vec::with_capacity(programs.len() + program_files.len());
    for (index, source) in programs.iter().enumerate() {
        compiled.push(
            compile_program_source(source)
                .with_context(|| format!("failed to compile --program #{}", index + 1))?,
        );
    }
    for path in program_files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read program file {}", path.display()))?;
        compiled.push(
            compile_program_source(&source)
                .with_context(|| format!("failed to compile program file {}", path.display()))?,
        );
    }
    Ok(compiled)
}

fn compile_program_source(source: &str) -> Result<ExecutableProgram> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("program source is empty"));
    }
    compile_executable_source(trimmed).map_err(|error| anyhow!("{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    #[test]
    fn load_initial_programs_from_inline_and_file() {
        let file = tempfile_program("while (mine()); dump();");
        let programs =
            load_initial_programs(&["mine();".to_string()], &[file.path().to_path_buf()])
                .expect("programs should compile");
        assert_eq!(programs.len(), 2);
        file.close().ok();
    }

    #[test]
    fn load_initial_programs_rejects_invalid_source() {
        let error = load_initial_programs(&["not a program!!!".to_string()], &[])
            .expect_err("invalid source should fail");
        assert!(error.to_string().contains("failed to compile --program #1"));
    }

    #[test]
    fn evaluate_only_requires_injected_programs() {
        let cli = Cli::try_parse_from(["robominer-optimize", "--evaluate-only"])
            .expect("cli should parse");
        assert!(cli.evaluate_only);
        assert!(cli.programs.is_empty());
        assert!(cli.program_files.is_empty());
    }

    #[test]
    fn parse_fixed_parts_requires_seven_ids() {
        use crate::catalog::PartCatalog;
        use robominer_db::RobotPartRecord;

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

        let catalog = PartCatalog::from_parts(
            (1..=7)
                .map(|type_id| sample_part(type_id * 10, type_id))
                .collect(),
            9,
        );
        assert!(parse_fixed_parts(&catalog, &[]).unwrap().is_none());
        assert!(parse_fixed_parts(&catalog, &[10, 20, 30]).is_err());
        assert!(parse_fixed_parts(&catalog, &[999, 20, 30, 40, 50, 60, 70]).is_err());
        let ids = parse_fixed_parts(&catalog, &[10, 20, 30, 40, 50, 60, 70])
            .expect("valid parts")
            .expect("some");
        assert_eq!(ids, [10, 20, 30, 40, 50, 60, 70]);
    }

    #[test]
    fn compile_program_source_rejects_empty() {
        let error = compile_program_source("   ").expect_err("empty");
        assert!(error.to_string().contains("empty"));
    }

    #[test]
    fn cli_parses_seeds_and_parts() {
        let cli = Cli::try_parse_from([
            "robominer-optimize",
            "--seeds",
            "3",
            "--parts",
            "10,20,30,40,50,60,70",
            "--program",
            "mine();",
            "--evaluate-only",
        ])
        .expect("cli should parse");
        assert_eq!(cli.seeds, Some(3));
        assert_eq!(cli.parts, vec![10, 20, 30, 40, 50, 60, 70]);
        assert!(cli.evaluate_only);
        assert_eq!(cli.programs, vec!["mine();"]);
    }

    #[test]
    fn mutate_parts_preserves_program() {
        use crate::catalog::PartCatalog;
        use robominer_db::RobotPartRecord;

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

        let parts = (1..=7)
            .flat_map(|type_id| {
                [
                    sample_part(type_id * 10, type_id),
                    sample_part(type_id * 10 + 1, type_id),
                ]
            })
            .collect();
        let catalog = PartCatalog::from_parts(parts, 9);
        let program = compile_executable_source("mine(); dump();").expect("compile");
        let mut rng = StdRng::seed_from_u64(7);
        let genome = Genome::with_program(&catalog, program.clone(), &mut rng);
        let mutated = genome.mutate_parts(&catalog, &mut rng);
        assert_eq!(mutated.program.actions(), program.actions());
    }

    struct TempProgram {
        path: PathBuf,
    }

    impl TempProgram {
        fn path(&self) -> &Path {
            &self.path
        }

        fn close(self) -> std::io::Result<()> {
            std::fs::remove_file(self.path)
        }
    }

    fn tempfile_program(source: &str) -> TempProgram {
        let path = std::env::temp_dir().join(format!(
            "robominer-optimize-program-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let mut file = std::fs::File::create(&path).expect("temp file");
        file.write_all(source.as_bytes()).expect("write temp");
        TempProgram { path }
    }
}

async fn connect_database(
    database_url: Option<String>,
    config: Option<std::path::PathBuf>,
) -> Result<robominer_db::MySqlPool> {
    let database_url =
        robominer_db::resolve_database_url(database_url, config.clone(), "robominer-optimize")
            .map_err(|error| anyhow!(error))?;

    let config_value = match robominer_db::load_legacy_config(config, "robominer-optimize") {
        Ok((_, config_map)) => {
            robominer_db::config_value(&config_map, "dbmaxconnections").map(str::to_owned)
        }
        Err(robominer_db::ConfigError::MissingConfigFile) => None,
        Err(error) => return Err(anyhow!(error)),
    };
    let max_connections = robominer_db::resolve_max_connections(
        std::env::var("ROBOMINER_DB_MAX_CONNECTIONS")
            .ok()
            .as_deref(),
        config_value.as_deref(),
    )
    .map_err(|error| anyhow!(error))?;

    robominer_db::connect_with_max_connections(&database_url, max_connections)
        .await
        .context("failed to connect to database")
}
