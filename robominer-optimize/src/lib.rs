mod catalog;
mod cli;
mod fitness;
mod ga;
mod genome;
mod report;

pub use catalog::PartCatalog;
pub use cli::Cli;
pub use fitness::{FitnessContext, FitnessResult, evaluate_genome};
pub use ga::{GaConfig, run_ga};
pub use genome::Genome;
pub use report::format_top_results;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use rand::SeedableRng;
use rand::rngs::StdRng;
use robominer_domain::load_mining_area_loadout;

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

    let mut rng = StdRng::seed_from_u64(cli.seed);
    let fitness_ctx = FitnessContext {
        areas: &areas,
        catalog: &catalog,
        depot_capacity: cli.depot_capacity,
        seeds: cli.seeds,
    };
    let ga_config = GaConfig {
        population: cli.population,
        generations: cli.generations,
        elite: cli.elite,
        mutation_rate: cli.mutation_rate,
        crossover_rate: cli.crossover_rate,
        tournament_size: cli.tournament_size,
    };

    let ranked = run_ga(&ga_config, &fitness_ctx, &mut rng);
    let report = format_top_results(&ranked, &catalog, cli.depot_capacity, cli.top_n);
    println!("{report}");
    Ok(())
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
