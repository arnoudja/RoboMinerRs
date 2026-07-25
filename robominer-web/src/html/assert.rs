//! Shared assertion helpers for tests that check rendered HTML fragments.
//!
//! These exist to replace bare `assert!(html.contains(...))` calls with
//! helpers that produce useful failure messages (including the actual HTML)
//! and that make the intent of the check ("contains", "does not contain",
//! "has this CSS class") explicit at the call site.

/// Asserts that `html` contains `fragment`, printing the full HTML on failure.
#[cfg(test)]
pub(crate) fn assert_html_contains(html: &str, fragment: &str) {
    assert!(
        html.contains(fragment),
        "expected HTML to contain `{fragment}`\nHTML:\n{html}"
    );
}

/// Asserts that `html` does not contain `fragment`, printing the full HTML on failure.
#[cfg(test)]
pub(crate) fn assert_html_not_contains(html: &str, fragment: &str) {
    assert!(
        !html.contains(fragment),
        "expected HTML to NOT contain `{fragment}`\nHTML:\n{html}"
    );
}

/// Asserts that `html` contains every fragment in `fragments`, in any order.
///
/// Reports every missing fragment (not just the first) so a single failure
/// gives the full picture instead of requiring repeated test runs.
#[cfg(test)]
pub(crate) fn assert_contains_all(html: &str, fragments: &[&str]) {
    let missing: Vec<&str> = fragments
        .iter()
        .copied()
        .filter(|fragment| !html.contains(fragment))
        .collect();
    assert!(
        missing.is_empty(),
        "expected HTML to contain all of {fragments:?}\nmissing: {missing:?}\nHTML:\n{html}"
    );
}

/// Asserts that `html` has an element carrying CSS class `class`.
///
/// Matches `class` whether it is the only class on the element or combined
/// with others, e.g. `class="foo"`, `class="foo bar"`, or `class="bar foo"`.
#[cfg(test)]
pub(crate) fn assert_html_has_class(html: &str, class: &str) {
    let is_only_class = html.contains(&format!("class=\"{class}\""));
    let is_last_class = html.contains(&format!(" {class}\""));
    let is_first_or_middle_class = html.contains(&format!("\"{class} ")) || {
        // matches a class in the middle of a multi-class attribute, e.g. `class="a foo b"`
        html.contains(&format!(" {class} "))
    };
    assert!(
        is_only_class || is_last_class || is_first_or_middle_class,
        "expected HTML to have an element with class `{class}`\nHTML:\n{html}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_html_contains_passes_for_present_fragment() {
        assert_html_contains("<p>hello</p>", "hello");
    }

    #[test]
    #[should_panic(expected = "expected HTML to contain")]
    fn assert_html_contains_panics_for_missing_fragment() {
        assert_html_contains("<p>hello</p>", "goodbye");
    }

    #[test]
    fn assert_html_not_contains_passes_for_absent_fragment() {
        assert_html_not_contains("<p>hello</p>", "goodbye");
    }

    #[test]
    #[should_panic(expected = "expected HTML to NOT contain")]
    fn assert_html_not_contains_panics_for_present_fragment() {
        assert_html_not_contains("<p>hello</p>", "hello");
    }

    #[test]
    fn assert_contains_all_passes_when_every_fragment_present() {
        assert_contains_all("<p>hello world</p>", &["hello", "world", "<p>"]);
    }

    #[test]
    #[should_panic(expected = "missing: [\"goodbye\"]")]
    fn assert_contains_all_reports_missing_fragments() {
        assert_contains_all("<p>hello world</p>", &["hello", "goodbye"]);
    }

    #[test]
    fn assert_html_has_class_matches_sole_class() {
        assert_html_has_class(r#"<div class="foo">"#, "foo");
    }

    #[test]
    fn assert_html_has_class_matches_last_of_multiple_classes() {
        assert_html_has_class(r#"<div class="bar foo">"#, "foo");
    }

    #[test]
    fn assert_html_has_class_matches_first_of_multiple_classes() {
        assert_html_has_class(r#"<div class="foo bar">"#, "foo");
    }

    #[test]
    fn assert_html_has_class_matches_middle_of_multiple_classes() {
        assert_html_has_class(r#"<div class="bar foo baz">"#, "foo");
    }

    #[test]
    #[should_panic(expected = "expected HTML to have an element with class")]
    fn assert_html_has_class_panics_when_class_absent() {
        assert_html_has_class(r#"<div class="bar">"#, "foo");
    }
}
