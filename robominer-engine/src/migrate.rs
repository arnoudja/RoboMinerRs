use anyhow::{Result, anyhow};
use robominer_db::{
    EMBEDDED_MIGRATIONS, MigrationReport, migration_status, run_embedded_migrations,
};

pub(crate) async fn migrate(pool: &robominer_db::MySqlPool) -> Result<()> {
    let report = run_embedded_migrations(pool)
        .await
        .map_err(|error| anyhow!(error))?;
    print_report(&report);
    Ok(())
}

pub(crate) async fn migrate_status(pool: &robominer_db::MySqlPool) -> Result<()> {
    let status = migration_status(pool, EMBEDDED_MIGRATIONS)
        .await
        .map_err(|error| anyhow!(error))?;
    for (version, applied) in status {
        let marker = if applied { "applied" } else { "pending" };
        println!("{version}\t{marker}");
    }
    Ok(())
}

fn print_report(report: &MigrationReport) {
    for version in &report.baselined {
        println!("{version}\tbaselined");
    }
    for version in &report.applied {
        println!("{version}\tapplied");
    }
    for version in &report.already_applied {
        println!("{version}\talready-applied");
    }
    if report.baselined.is_empty() && report.applied.is_empty() && report.already_applied.is_empty()
    {
        println!("no-migrations");
    }
}
