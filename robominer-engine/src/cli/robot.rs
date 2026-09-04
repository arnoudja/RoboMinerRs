use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum RobotCommand {
    ConfigStates {
        #[arg(long)]
        user_id: i64,
    },
    UpdateConfig {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_id: i64,

        #[arg(long)]
        robot_name: String,

        #[arg(long)]
        program_source_id: i64,

        #[arg(long)]
        ore_container_id: i64,

        #[arg(long)]
        mining_unit_id: i64,

        #[arg(long)]
        battery_id: i64,

        #[arg(long)]
        memory_module_id: i64,

        #[arg(long)]
        cpu_id: i64,

        #[arg(long)]
        engine_id: i64,

        #[arg(long)]
        ore_scanner_id: i64,
    },
}
