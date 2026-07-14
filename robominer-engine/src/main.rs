use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    robominer_engine::run().await
}
