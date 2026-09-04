//! Shared percent-encoding for cookies and query components.

/// Percent-encode bytes that are not alphanumeric and not in `extra_unescaped`.
pub(crate) fn percent_encode(value: &str, extra_unescaped: &[u8]) -> String {
    value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || extra_unescaped.contains(&byte) {
                vec![byte as char]
            } else {
                format!("%{byte:02X}").chars().collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::percent_encode;

    #[test]
    fn percent_encode_leaves_extra_unescaped() {
        assert_eq!(percent_encode("a b@c", b"-_.@"), "a%20b@c");
        assert_eq!(percent_encode("a b~c", b"-_.~"), "a%20b~c");
    }
}
