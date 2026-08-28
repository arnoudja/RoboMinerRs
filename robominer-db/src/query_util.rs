//! Shared SQL query helpers (dynamic `IN` lists, etc.).

/// Build a comma-separated list of `?` placeholders for SQL `IN` clauses.
pub fn in_placeholders(count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    vec!["?"; count].join(", ")
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
