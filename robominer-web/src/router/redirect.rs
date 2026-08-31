use crate::Request;
use crate::Response;
use crate::routes::AppRoute;

/// Redirect legacy PascalCase paths (`/Shop`, `/MiningQueue`, …) to canonical camelCase.
///
/// GET/HEAD only: mutating POSTs must reach the handler directly so form bodies and
/// CSRF tokens are not dropped by a redirect.
pub(super) fn canonical_path_redirect(request: &Request) -> Option<Response> {
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        return None;
    }

    let canonical = canonicalize_path(&request.path)?;
    if canonical == request.path {
        return None;
    }

    log_legacy_path_redirect(request, &canonical);

    let mut location = canonical;
    if !request.query.is_empty() {
        let mut pairs: Vec<_> = request.query.iter().collect();
        pairs.sort_by_key(|(key, _)| *key);
        location.push('?');
        for (index, (key, value)) in pairs.into_iter().enumerate() {
            if index > 0 {
                location.push('&');
            }
            location.push_str(key);
            location.push('=');
            location.push_str(value);
        }
    }

    Some(Response::redirect(location))
}

fn log_legacy_path_redirect(request: &Request, canonical: &str) {
    tracing::info!(
        legacy_path = %request.path,
        canonical_path = %canonical,
        method = %request.method,
        "legacy_pascal_case_redirect"
    );
}

pub(super) fn canonicalize_path(path: &str) -> Option<String> {
    // Prefer the typed route table for known PascalCase aliases so redirect
    // targets stay aligned with dispatch. Extra aliases (legacy `.html` paths)
    // are left alone here — those still resolve via dispatch matching.
    if let Some(canonical) = AppRoute::canonicalize(path) {
        let route = AppRoute::from_path(path)?;
        if path == route.pascal_path() {
            return Some(canonical.to_string());
        }
        return None;
    }

    let rest = path.strip_prefix('/')?;
    let mut chars = rest.chars();
    let first = chars.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    Some(format!("/{}{}", first.to_ascii_lowercase(), chars.as_str()))
}
