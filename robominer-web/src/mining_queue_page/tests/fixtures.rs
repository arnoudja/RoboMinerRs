//! Shared fixtures for `mining_queue_page` unit tests.

use std::collections::HashMap;

use crate::Request;
use crate::session::format_authenticated_cookie;

pub(super) fn authenticated_request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::from([(
            "cookie".to_string(),
            format_authenticated_cookie(42, "Player"),
        )]),
    }
}
