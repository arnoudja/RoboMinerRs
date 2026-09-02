//! Form and query parameter parsing helpers.

use crate::http::Request;

pub(crate) fn is_post(request: &Request) -> bool {
    request.method.eq_ignore_ascii_case("POST")
}

pub(crate) fn query_i64(request: &Request, name: &str) -> Option<i64> {
    request
        .query
        .get(name)
        .or_else(|| request.form.get(name))
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
}

pub(crate) fn query_signed_i64(request: &Request, name: &str) -> Option<i64> {
    request
        .query
        .get(name)
        .or_else(|| request.form.get(name))
        .and_then(|value| value.parse::<i64>().ok())
}

/// Positive integer from the POST form body only (ignores query string).
pub(crate) fn form_i64(request: &Request, name: &str) -> Option<i64> {
    request
        .form
        .get(name)
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
}

/// State-changing id parameters: POST form only.
pub(crate) fn mutation_i64(request: &Request, name: &str) -> Option<i64> {
    if is_post(request) {
        form_i64(request, name)
    } else {
        None
    }
}

pub(crate) fn mutation_form_has(request: &Request, name: &str) -> bool {
    is_post(request) && request.form.contains_key(name)
}
