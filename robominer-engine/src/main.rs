//! Binary entry for `robominer-engine`.
//!
//! Initializes tracing, then hands off to [`robominer_engine::run`] which parses
//! the CLI and dispatches to mining/shop/user/rally/migrate (and related) commands.
//! Operators usually set `ROBOMINER_DATABASE_URL` or pass `--database-url`.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    robominer_db::init_default_tracing();
    robominer_engine::run().await
}
