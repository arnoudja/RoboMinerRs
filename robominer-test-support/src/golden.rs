use std::fs;
use std::path::{Path, PathBuf};

pub fn update_golden_enabled(env_var: &str) -> bool {
    std::env::var(env_var).is_ok()
}

pub fn round_golden_score(score: f64) -> f64 {
    (score * 1_000_000.0).round() / 1_000_000.0
}

pub fn round_golden_coord(value: f64) -> f64 {
    (value * 1_000_000_000.0).round() / 1_000_000_000.0
}

pub fn fixture_path(manifest_dir: &str, subdir: &str, name: &str) -> PathBuf {
    Path::new(manifest_dir).join(format!("tests/fixtures/{subdir}/{name}.json"))
}

pub fn load_fixture<T: serde::de::DeserializeOwned>(
    manifest_dir: &str,
    subdir: &str,
    name: &str,
) -> T {
    let path = fixture_path(manifest_dir, subdir, name);
    let contents = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read golden fixture {}: {error}", path.display())
    });
    serde_json::from_str(&contents).unwrap_or_else(|error| {
        panic!("failed to parse golden fixture {}: {error}", path.display())
    })
}

pub fn write_fixture<T: serde::Serialize>(
    manifest_dir: &str,
    subdir: &str,
    name: &str,
    fixture: &T,
) {
    let path = fixture_path(manifest_dir, subdir, name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory should be creatable");
    }
    let contents = serde_json::to_string_pretty(fixture).expect("fixture should serialize");
    fs::write(&path, contents).expect("fixture should be writable");
}

/// Write or compare golden fixtures for each scenario id.
///
/// When `update_env_var` is set in the environment, writes one JSON fixture per
/// scenario (the first element of `build`'s return value). Otherwise loads the
/// stored fixture and invokes `compare` with the built fixture and extra context.
pub fn assert_or_update_golden<S, T, E, FBuild, FCompare>(
    update_env_var: &str,
    manifest_dir: &str,
    fixture_subdir: &str,
    scenarios: &[S],
    fixture_name: impl Fn(S) -> &'static str,
    build: FBuild,
    compare: FCompare,
) where
    S: Copy,
    T: serde::Serialize + serde::de::DeserializeOwned,
    FBuild: Fn(S) -> (T, E),
    FCompare: Fn(S, T, T, E),
{
    if update_golden_enabled(update_env_var) {
        for scenario in scenarios {
            let name = fixture_name(*scenario);
            let (fixture, _) = build(*scenario);
            write_fixture(manifest_dir, fixture_subdir, name, &fixture);
        }
        return;
    }

    for scenario in scenarios {
        let name = fixture_name(*scenario);
        let expected: T = load_fixture(manifest_dir, fixture_subdir, name);
        let (actual, extra) = build(*scenario);
        compare(*scenario, expected, actual, extra);
    }
}

/// Async counterpart of [`assert_or_update_golden`] for scenarios whose
/// fixture build requires awaiting (e.g. database-backed setup/build/cleanup).
pub async fn assert_or_update_golden_async<S, T, E, FBuild, FCompare, Fut>(
    update_env_var: &str,
    manifest_dir: &str,
    fixture_subdir: &str,
    scenarios: &[S],
    fixture_name: impl Fn(S) -> &'static str,
    build: FBuild,
    compare: FCompare,
) where
    S: Copy,
    T: serde::Serialize + serde::de::DeserializeOwned,
    FBuild: Fn(S) -> Fut,
    Fut: std::future::Future<Output = (T, E)>,
    FCompare: Fn(S, T, T, E),
{
    if update_golden_enabled(update_env_var) {
        for scenario in scenarios {
            let name = fixture_name(*scenario);
            let (fixture, _) = build(*scenario).await;
            write_fixture(manifest_dir, fixture_subdir, name, &fixture);
        }
        return;
    }

    for scenario in scenarios {
        let name = fixture_name(*scenario);
        let expected: T = load_fixture(manifest_dir, fixture_subdir, name);
        let (actual, extra) = build(*scenario).await;
        compare(*scenario, expected, actual, extra);
    }
}
