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
}
