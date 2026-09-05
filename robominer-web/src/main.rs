use std::io;
use std::net::TcpListener;

use clap::Parser;
use robominer_web::serve;
use robominer_web::startup::{connect_database, prepare_server_config};

#[derive(Debug, Parser)]
#[command(name = "robominer-web")]
#[command(about = "Rust web host for RoboMiner")]
struct Cli {
    #[arg(long)]
    database_url: Option<String>,
}

fn main() -> io::Result<()> {
    robominer_db::init_default_tracing();

    let cli = Cli::parse();
    let database_pool = connect_database(cli.database_url)?;
    let (host, port, server_config) = prepare_server_config(database_pool)?;

    let listener = TcpListener::bind(format!("{host}:{port}"))?;
    tracing::info!(
        addr = %listener.local_addr()?,
        static_root = %server_config.static_root.display(),
        "robominer-web listening"
    );

    serve(listener, server_config)
}
