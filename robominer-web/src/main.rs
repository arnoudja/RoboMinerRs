use std::io;
use std::net::TcpListener;
use std::path::PathBuf;

use clap::Parser;
use robominer_web::serve;
use robominer_web::startup::{
    connect_database, default_web_root, load_legacy_config, prepare_server_config,
};

#[derive(Debug, Parser)]
#[command(name = "robominer-web")]
#[command(about = "Rust web host for RoboMiner")]
struct Cli {
    #[arg(long)]
    database_url: Option<String>,

    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let _default_static_root = default_web_root();
    let (config_path, legacy_config) = load_legacy_config(cli.config.clone())?;
    let database_pool = connect_database(cli.database_url, cli.config, &legacy_config)?;
    let (host, port, server_config) = prepare_server_config(&legacy_config, database_pool)?;

    let listener = TcpListener::bind(format!("{host}:{port}"))?;
    eprintln!(
        "robominer-web listening on http://{} with static root {} (config {})",
        listener.local_addr()?,
        server_config.static_root.display(),
        config_path.display()
    );

    serve(listener, server_config)
}
