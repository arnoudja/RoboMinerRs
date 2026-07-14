use robominer_db::{
    CompletedPoolItemOreRecord, CompletedPoolItemRecord, CompletedPoolRallyRecord,
    persist_completed_pool_rally,
};
use robominer_test_support::PoolFixture;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn persist_completed_pool_rally_updates_scores_and_ore_totals() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db pool test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = PoolFixture::create(&pool).await;

    persist_completed_pool_rally(
        &pool,
        &CompletedPoolRallyRecord {
            items: vec![CompletedPoolItemRecord {
                pool_item_id: fixture.pool_item_id,
                score: 7.25,
                ore_results: vec![
                    CompletedPoolItemOreRecord {
                        ore_id: fixture.ore_id,
                        amount: 4,
                    },
                    CompletedPoolItemOreRecord {
                        ore_id: fixture.ore_id,
                        amount: 0,
                    },
                ],
            }],
        },
    )
    .await
    .expect("persist should succeed");

    fixture.assert_persisted(&pool).await;
    fixture.cleanup(&pool).await;
}
