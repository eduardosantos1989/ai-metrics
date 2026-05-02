use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_initial_commands() {
    let mut cmd = Command::cargo_bin("ai-ledger").expect("binary exists");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Privacy-first AI observability"));
}

#[test]
fn init_creates_local_ledger_layout() {
    let temp = tempfile::tempdir().expect("tempdir");

    let mut cmd = Command::cargo_bin("ai-ledger").expect("binary exists");
    cmd.args(["init", "--path"])
        .arg(temp.path())
        .assert()
        .success();

    assert!(temp.path().join(".ai-ledger/config.toml").exists());
    assert!(temp.path().join(".ai-ledger/events").is_dir());
    assert!(temp.path().join(".ai-ledger/manifests").is_dir());
    assert!(temp.path().join(".ai-ledger/reports").is_dir());
    assert!(temp.path().join(".ai-ledger/baselines").is_dir());
}

#[test]
fn append_and_replay_event_log() {
    let temp = tempfile::tempdir().expect("tempdir");
    let log = temp.path().join("events.jsonl");

    let mut append = Command::cargo_bin("ai-ledger").expect("binary exists");
    append
        .args(["event", "append", "--run-id", "run_cli", "--log"])
        .arg(&log)
        .args(["--tag", "service=cli-test"])
        .assert()
        .success();

    let mut replay = Command::cargo_bin("ai-ledger").expect("binary exists");
    replay
        .args(["event", "replay", "--log"])
        .arg(&log)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"event_type\":\"llm_request\""))
        .stdout(predicate::str::contains("\"service\":\"cli-test\""));
}
