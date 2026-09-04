use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub(crate) enum ProgramCommand {
    Verify {
        program_source_id: i64,
    },
    VerifySource {
        source_file: PathBuf,
    },
    SimulateSource {
        source_file: Option<PathBuf>,

        #[arg(long)]
        robot: Vec<PathBuf>,

        #[arg(long, default_value_t = 10)]
        turns: i32,

        #[arg(long, default_value_t = 5)]
        size_x: usize,

        #[arg(long, default_value_t = 5)]
        size_y: usize,

        #[arg(long, default_value_t = 1)]
        ore_x: usize,

        #[arg(long, default_value_t = 1)]
        ore_y: usize,

        #[arg(long, default_value_t = 0)]
        ore_type: usize,

        #[arg(long, default_value_t = 8)]
        ore_amount: i32,

        #[arg(long, default_value_t = 4)]
        mining_speed: i32,

        #[arg(long, default_value_t = 1.5)]
        forward_speed: f64,

        #[arg(long, default_value_t = 1.0)]
        backward_speed: f64,

        #[arg(long, default_value_t = 90)]
        rotate_speed: i32,
    },
    CreateSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        source_name: String,

        #[arg(long)]
        source_code: String,
    },
    UpdateSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        program_source_id: i64,

        #[arg(long)]
        source_name: String,

        #[arg(long)]
        source_code: String,
    },
    DeleteSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        program_source_id: i64,
    },
    SourceStates {
        #[arg(long)]
        user_id: i64,
    },
}
