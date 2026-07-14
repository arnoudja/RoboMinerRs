pub(crate) fn escape_state_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

pub(crate) fn optional_id(value: Option<i64>) -> String {
    value.map(|id| id.to_string()).unwrap_or_default()
}
