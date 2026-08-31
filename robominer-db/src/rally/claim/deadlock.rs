/// InnoDB deadlock SQLSTATE (`ER_LOCK_DEADLOCK` / MySQL 1213). Restart the claim
/// transaction; one concurrent worker still wins the row locks.
pub(super) const MYSQL_DEADLOCK_SQLSTATE: &str = "40001";
pub(super) const MAX_CLAIM_DEADLOCK_ATTEMPTS: u32 = 8;

pub(super) fn is_mysql_deadlock(error: &sqlx::Error) -> bool {
    is_mysql_deadlock_sqlstate(
        error
            .as_database_error()
            .and_then(|database_error| database_error.code())
            .as_deref(),
    )
}

pub(super) fn is_mysql_deadlock_sqlstate(code: Option<&str>) -> bool {
    code == Some(MYSQL_DEADLOCK_SQLSTATE)
}

#[cfg(test)]
mod tests {
    use super::{is_mysql_deadlock, is_mysql_deadlock_sqlstate};

    #[test]
    fn mysql_deadlock_sqlstate_matches_innodb_restart_code() {
        assert!(is_mysql_deadlock_sqlstate(Some("40001")));
        assert!(!is_mysql_deadlock_sqlstate(Some("HY000")));
        assert!(!is_mysql_deadlock_sqlstate(None));
        assert!(!is_mysql_deadlock(&sqlx::Error::RowNotFound));
        assert!(!is_mysql_deadlock(&sqlx::Error::PoolClosed));
    }
}
