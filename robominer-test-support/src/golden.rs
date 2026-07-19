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
