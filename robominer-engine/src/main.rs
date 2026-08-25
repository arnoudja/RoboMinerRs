use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    robominer_db::init_default_tracing();
    robominer_engine::run().await
}
