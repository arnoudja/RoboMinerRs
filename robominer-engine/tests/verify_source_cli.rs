use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempSource {
    path: PathBuf,
}

impl TempSource {
    fn new(name: &str, contents: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "robominer-{name}-{}-{unique}.txt",
            std::process::id()
        ));

        fs::write(&path, contents).expect("failed to write temporary source file");

        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempSource {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn run_engine(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_robominer-engine"))
        .args(args)
        .output()
        .expect("failed to execute robominer-engine")
}

fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn verify_source_accepts_valid_program_without_database_config() {
    let source = TempSource::new("valid", "move(1);\nmine();\n");
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "verify-source",
        source
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected success, got status {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code()
    );
    assert!(
        stdout.contains("verified, compiled size 4"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}

#[test]
fn verify_source_reports_invalid_program() {
    let source = TempSource::new("invalid", "move(1) mine();\n");
    let output = run_engine(&[
        "verify-source",
        source
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected invalid source to be reported without a process failure\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("invalid:"), "unexpected stdout:\n{stdout}");
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}

#[test]
fn verify_source_fails_when_file_cannot_be_read() {
    let missing_path = std::env::temp_dir().join(format!(
        "robominer-missing-source-{}-{}.txt",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    let output = run_engine(&[
        "verify-source",
        missing_path
            .to_str()
            .expect("temporary path should be UTF-8"),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected missing source file to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("failed to read source file"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn simulate_source_runs_valid_program_without_database_config() {
    let source = TempSource::new("simulate-valid", "move(1.4142135623730951); mine();\n");
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "simulate-source",
        "--turns",
        "2",
        source
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected success, got status {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code()
    );
    assert!(
        stdout.contains("Simulation complete"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stdout.contains("robots: 1"), "unexpected stdout:\n{stdout}");
    assert!(
        stdout.contains("robot 1 position:"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("robot 1 score:"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("forward=1") && stdout.contains("mine=1"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}

#[test]
fn simulate_source_fails_for_invalid_program() {
    let source = TempSource::new("simulate-invalid", "move(1) mine();\n");
    let output = run_engine(&[
        "simulate-source",
        source
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected invalid executable source to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("failed to compile executable program"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn simulate_source_runs_multiple_robot_files() {
    let first = TempSource::new("simulate-robot-1", "move(1);\n");
    let second = TempSource::new("simulate-robot-2", "move(1);\n");
    let output = run_engine(&[
        "simulate-source",
        "--turns",
        "1",
        "--robot",
        first
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
        "--robot",
        second
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected success, got status {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code()
    );
    assert!(stdout.contains("robots: 2"), "unexpected stdout:\n{stdout}");
    assert!(
        stdout.contains("robot 1 source:") && stdout.contains("robot 2 source:"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("distance robot 1-2:"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}

#[test]
fn simulate_source_rejects_positional_and_robot_files_together() {
    let positional = TempSource::new("simulate-positional", "move(1);\n");
    let robot = TempSource::new("simulate-robot", "move(1);\n");
    let output = run_engine(&[
        "simulate-source",
        positional
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
        "--robot",
        robot
            .path()
            .to_str()
            .expect("temporary path should be UTF-8"),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected mixed source forms to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("provide either a positional source file or --robot files"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_rally_help_documents_persist_mode() {
    let output = run_engine(&["run-rally", "--help"]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected help to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("--persist"), "unexpected stdout:\n{stdout}");
    assert!(
        stdout.contains("--result-data-file"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}

#[test]
fn run_pool_help_documents_dry_run_and_persist_mode() {
    let output = run_engine(&["run-pool", "--help"]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected help to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("--pool-id"), "unexpected stdout:\n{stdout}");
    assert!(stdout.contains("--seed"), "unexpected stdout:\n{stdout}");
    assert!(stdout.contains("--persist"), "unexpected stdout:\n{stdout}");
    assert!(
        stdout.contains("--until-complete"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("--max-rallies"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}

#[test]
fn run_pool_rejects_non_positive_pool_id_before_database_config() {
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "run-pool",
        "--pool-id",
        "0",
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected invalid pool id to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("--pool-id must be greater than zero"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_pool_until_complete_requires_persist() {
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "run-pool",
        "--pool-id",
        "1",
        "--until-complete",
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected until-complete without persist to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("--until-complete requires --persist"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_pool_rejects_zero_max_rallies() {
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "run-pool",
        "--pool-id",
        "1",
        "--persist",
        "--until-complete",
        "--max-rallies",
        "0",
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected zero max-rallies to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("--max-rallies must be greater than zero"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_rallies_requires_once_or_loop() {
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "run-rallies",
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected missing --once to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("run-rallies requires exactly one of --once or --loop"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_rallies_rejects_once_and_loop_together() {
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "run-rallies",
        "--once",
        "--loop",
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected conflicting modes to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("run-rallies requires exactly one of --once or --loop"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_rallies_loop_requires_persist() {
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "run-rallies",
        "--loop",
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected loop without persist to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("--loop requires --persist"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_rallies_rejects_zero_sleep_seconds() {
    let output = run_engine(&[
        "--config",
        "/tmp/robominer-engine-config-that-should-not-be-read.conf",
        "run-rallies",
        "--once",
        "--sleep-seconds",
        "0",
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected zero sleep seconds to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("--sleep-seconds must be greater than zero"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn run_rallies_help_documents_once_and_persist() {
    let output = run_engine(&["run-rallies", "--help"]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected help to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("--once"), "unexpected stdout:\n{stdout}");
    assert!(stdout.contains("--loop"), "unexpected stdout:\n{stdout}");
    assert!(
        stdout.contains("--sleep-seconds"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stdout.contains("--persist"), "unexpected stdout:\n{stdout}");
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}
