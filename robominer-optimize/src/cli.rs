use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "robominer-optimize")]
#[command(about = "Genetic optimizer for robot parts and programs (read-only DB)")]
pub struct Cli {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Comma-separated mining area ids
    #[arg(long, value_delimiter = ',', default_value = "1001,1002")]
    pub areas: Vec<i64>,

    /// Maximum RobotPart.tierId allowed in the evolved loadout
    #[arg(long, default_value_t = 9999)]
    pub max_tier_id: i64,

    /// Fixed depot capacity for ore slot A (Cerbonium); not evolved
    #[arg(long, default_value_t = 40)]
    pub depot_capacity: i32,

    #[arg(long, default_value_t = 40)]
    pub population: usize,

    #[arg(long, default_value_t = 50)]
    pub generations: usize,

    /// Number of rally seeds (0..seeds-1) averaged per area
    #[arg(long, default_value_t = 5)]
    pub seeds: u64,

    #[arg(long, default_value_t = 2)]
    pub elite: usize,

    #[arg(long, default_value_t = 0.2)]
    pub mutation_rate: f64,

    #[arg(long, default_value_t = 0.7)]
    pub crossover_rate: f64,

    #[arg(long, default_value_t = 3)]
    pub tournament_size: usize,

    #[arg(long, default_value_t = 5)]
    pub top_n: usize,

    /// RNG seed for reproducibility
    #[arg(long, default_value_t = 1)]
    pub seed: u64,
}
