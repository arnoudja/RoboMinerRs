//! Shared SQL query helpers (dynamic `IN` lists, etc.).

use sqlx::AssertSqlSafe;

/// Build a comma-separated list of `?` placeholders for SQL `IN` clauses.
pub fn in_placeholders(count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    vec!["?"; count].join(", ")
}

/// Mark a dynamically built SQL string as audited for injection safety.
///
/// Use only when the dynamic fragments are structural (placeholder counts,
/// fixed column/table fragments from constants) and all user values are bound.
pub fn assert_sql_safe(sql: String) -> AssertSqlSafe<String> {
    AssertSqlSafe(sql)
}

#[cfg(test)]
mod tests {
    use super::in_placeholders;

    #[test]
    fn in_placeholders_builds_comma_separated_question_marks() {
        assert_eq!(in_placeholders(0), "");
        assert_eq!(in_placeholders(1), "?");
        assert_eq!(in_placeholders(3), "?, ?, ?");
    }
}
