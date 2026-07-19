use anyhow::{Result, anyhow, ensure};
use robominer_db::{
    EMBEDDED_MIGRATIONS, MigrationReport, migration_status, run_embedded_migrations,
};

pub(crate) async fn migrate(pool: &robominer_db::MySqlPool) -> Result<()> {
    let report = run_embedded_migrations(pool)
        .await
        .map_err(|error| anyhow!(error))?;
    print!("{}", format_report(&report));
    Ok(())
}

pub(crate) async fn migrate_status(pool: &robominer_db::MySqlPool, check: bool) -> Result<()> {
    let status = migration_status(pool, EMBEDDED_MIGRATIONS)
        .await
        .map_err(|error| anyhow!(error))?;
    print!("{}", format_status(&status));
    if check {
        let pending: Vec<&str> = status
            .iter()
            .filter(|(_, applied)| !*applied)
            .map(|(version, _)| version.as_str())
            .collect();
        ensure!(
            pending.is_empty(),
            "pending schema migrations: {}",
            pending.join(", ")
        );
    }
    Ok(())
}

fn format_report(report: &MigrationReport) -> String {
    let mut out = String::new();
    for version in &report.baselined {
        out.push_str(&format!("{version}\tbaselined\n"));
    }
    for version in &report.applied {
        out.push_str(&format!("{version}\tapplied\n"));
    }
    for version in &report.already_applied {
        out.push_str(&format!("{version}\talready-applied\n"));
    }
    if report.baselined.is_empty() && report.applied.is_empty() && report.already_applied.is_empty()
    {
        out.push_str("no-migrations\n");
    }
    out
}

fn format_status(status: &[(String, bool)]) -> String {
    let mut out = String::new();
    for (version, applied) in status {
        let marker = if *applied { "applied" } else { "pending" };
        out.push_str(&format!("{version}\t{marker}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_report_covers_all_buckets() {
        let report = MigrationReport {
            baselined: vec!["001_a".into()],
            applied: vec!["002_b".into()],
            already_applied: vec!["003_c".into()],
        };
        assert_eq!(
            format_report(&report),
            "001_a\tbaselined\n002_b\tapplied\n003_c\talready-applied\n"
        );
    }

    #[test]
    fn format_report_empty_prints_no_migrations() {
        let report = MigrationReport {
            baselined: vec![],
            applied: vec![],
            already_applied: vec![],
        };
        assert_eq!(format_report(&report), "no-migrations\n");
    }

    #[test]
    fn format_status_marks_applied_and_pending() {
        let status = vec![("001_a".into(), true), ("002_b".into(), false)];
        assert_eq!(format_status(&status), "001_a\tapplied\n002_b\tpending\n");
    }
}
