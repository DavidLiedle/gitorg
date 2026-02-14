use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_shows_all_commands() {
    let mut cmd = Command::cargo_bin("gitorg").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("orgs"))
        .stdout(predicate::str::contains("repos"))
        .stdout(predicate::str::contains("stale"))
        .stdout(predicate::str::contains("issues"))
        .stdout(predicate::str::contains("stats"))
        .stdout(predicate::str::contains("overview"));
}

#[test]
fn version_flag() {
    let mut cmd = Command::cargo_bin("gitorg").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("gitorg"));
}

#[test]
fn auth_help() {
    let mut cmd = Command::cargo_bin("gitorg").unwrap();
    cmd.args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("token"));
}

#[test]
fn repos_help_shows_sort() {
    let mut cmd = Command::cargo_bin("gitorg").unwrap();
    cmd.args(["repos", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--sort"));
}

#[test]
fn stale_help_shows_days() {
    let mut cmd = Command::cargo_bin("gitorg").unwrap();
    cmd.args(["stale", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--days"));
}

#[test]
fn no_subcommand_shows_help() {
    let mut cmd = Command::cargo_bin("gitorg").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn orgs_without_auth_fails() {
    let mut cmd = Command::cargo_bin("gitorg").unwrap();
    // Use a temp config dir to ensure no real auth exists
    cmd.env("XDG_CONFIG_HOME", "/tmp/gitorg_test_nonexistent")
        .arg("orgs")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not authenticated"));
}
