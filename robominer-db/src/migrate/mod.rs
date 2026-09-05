//! Schema migration loader and runner (embedded + on-disk).
//!
//! Primary entry points: [`run_embedded_migrations`], [`migration_status`],
//! [`run_migrations_from_dir`].

mod errors;
mod loader;
mod runner;
mod schema;
mod special;

include!(concat!(env!("OUT_DIR"), "/embedded_migrations.rs"));

pub use errors::MigrateError;
pub use loader::{default_migrations_dir, load_migrations_from_dir, split_sql_statements};
pub use runner::{
    MigrationReport, migration_status, run_embedded_migrations, run_migrations,
    run_migrations_from_dir,
};

#[cfg(test)]
mod tests {
    use super::{EMBEDDED_MIGRATIONS, load_migrations_from_dir, split_sql_statements};

    /// Migrations that introduce a distinctive marker checked by
    /// `schema_already_current`. Versions not listed are covered by a later
    /// probe once createDatabase.sql includes them (002, 004, 007 today).
    const BASELINE_PROBE_MIGRATIONS: &[&str] = &[
        "001_rename_scan_speed_to_scan_time",
        "003_user_session_version",
        "005_mining_area_score_ore_target",
        "006_ai_robot_table",
        "008_container_and_depot_tax_rates",
        "009_mining_area_lifetime_total_runs",
        "010_achievement_step_depot_total_requirement",
        "011_mining_queue_processing_lease",
        "012_mining_queue_claimable_index",
        "013_robot_lifetime_depot_amount",
    ];

    /// Migrations intentionally without their own probe (covered by later markers).
    const BASELINE_COVERED_BY_LATER_PROBE: &[&str] = &[
        "002_mining_queue_executed_source_code",
        "004_ore_depot_capacity",
        "007_ai_robot_depot_size",
    ];

    #[test]
    fn embedded_migrations_match_filesystem() {
        let dir = super::default_migrations_dir();
        let from_disk = load_migrations_from_dir(&dir).expect("load migrations dir");
        assert_eq!(from_disk.len(), EMBEDDED_MIGRATIONS.len());
        for ((disk_version, disk_sql), (embedded_version, embedded_sql)) in
            from_disk.iter().zip(EMBEDDED_MIGRATIONS.iter())
        {
            assert_eq!(disk_version, embedded_version);
            assert_eq!(
                split_sql_statements(disk_sql),
                split_sql_statements(embedded_sql)
            );
        }
    }

    #[test]
    fn every_embedded_migration_has_baseline_probe_policy() {
        for (version, _) in EMBEDDED_MIGRATIONS {
            let probed = BASELINE_PROBE_MIGRATIONS.contains(version);
            let deferred = BASELINE_COVERED_BY_LATER_PROBE.contains(version);
            assert!(
                probed ^ deferred,
                "migration {version} must be listed in exactly one of \
                 BASELINE_PROBE_MIGRATIONS or BASELINE_COVERED_BY_LATER_PROBE \
                 (update schema_already_current + migrate-database.sh when adding probes)"
            );
        }

        for version in BASELINE_PROBE_MIGRATIONS
            .iter()
            .chain(BASELINE_COVERED_BY_LATER_PROBE.iter())
        {
            assert!(
                EMBEDDED_MIGRATIONS
                    .iter()
                    .any(|(embedded, _)| embedded == version),
                "stale baseline policy entry {version} is not an embedded migration"
            );
        }
    }

    #[test]
    fn baseline_probe_markers_appear_in_schema_shell_and_create_sql() {
        // Keep in lockstep with resources/scripts/check-migration-baseline-sync.sh.
        let schema = include_str!("schema.rs");
        let shell = include_str!("../../../resources/scripts/migrate-database.sh");
        let create_sql = include_str!("../../../resources/database/createDatabase.sql");

        let required = [
            "scanTime",
            "sessionVersion",
            "scoreOreTarget",
            "AIRobot",
            "depotTaxRate",
            "MiningOreResult",
            "depotAmount",
            "MiningAreaLifetimeResult",
            "totalRuns",
            "AchievementStepDepotTotalRequirement",
            "processingLeaseUntil",
            "idx_mining_queue_claimable",
            "RobotLifetimeResult",
        ];
        for marker in required {
            assert!(
                schema.contains(marker),
                "schema.rs missing baseline marker {marker}"
            );
            assert!(
                shell.contains(marker),
                "migrate-database.sh missing baseline marker {marker}"
            );
            assert!(
                create_sql.contains(marker),
                "createDatabase.sql missing baseline marker {marker}"
            );
        }

        assert!(
            schema.contains("scanSpeed") && shell.contains("scanSpeed"),
            "scanSpeed absence probe must remain in schema.rs and migrate-database.sh"
        );
        assert!(
            !create_sql.contains(" scanSpeed ") && !create_sql.contains("\tscanSpeed "),
            "createDatabase.sql must not define legacy scanSpeed column"
        );
    }
}
