use assert_cmd::prelude::*;
use predicates::str::{contains, is_empty};
use std::fs::{self, File};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// `kvs-client` with no args should exit with a non-zero code.
#[test]
fn client_cli_no_args() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-client").unwrap();
    cmd.current_dir(&temp_dir).assert().failure();
}

#[test]
fn client_cli_invalid_get() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "extra", "field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_set() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "missing_field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "key", "value", "extra_field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "key", "value", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_rm() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["rm"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["rm", "extra", "field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["rm", "key", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["rm", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_subcommand() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["unknown"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

// `kvs-client -V` should print the version
#[test]
fn client_cli_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-client").unwrap();
    cmd.args(&["-V"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

// `kvs-server -V` should print the version
#[test]
fn server_cli_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-server").unwrap();
    cmd.args(&["-V"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_log_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let stderr_path = temp_dir.path().join("stderr");
    let mut cmd = Command::cargo_bin("kvs-server").unwrap();
    let mut child = cmd
        .args(&["--engine", "kvs", "--addr", "127.0.0.1:4001"])
        .current_dir(&temp_dir)
        .stderr(File::create(&stderr_path).unwrap())
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_secs(1));
    child.kill().expect("server exited before killed");
}

fn cli_access_server(engine: &str, addr: &str) {
    let (sender, receiver) = mpsc::sync_channel(0);
    let temp_dir = TempDir::new().unwrap();
    let mut server = Command::cargo_bin("kvs-server").unwrap();
    let mut child = server
        .args(&["--engine", engine, "--addr", addr])
        .current_dir(&temp_dir)
        .spawn()
        .unwrap();
    let handle = thread::spawn(move || {
        let _ = receiver.recv(); // wait for main thread to finish
        child.kill().expect("server exited before killed");
    });
    thread::sleep(Duration::from_secs(1));

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "set", "key1", "value1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "get", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("value1\n");

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "set", "key1", "value2"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "get", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("value2\n");

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "get", "key2"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "rm", "key2"])
        .current_dir(&temp_dir);

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "set", "key2", "value3"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["--addr", addr, "rm", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());

    sender.send(()).unwrap();
    handle.join().unwrap();
}
#[test]
fn cli_access_server_kvs_engine() {
    cli_access_server("kvs", "127.0.0.1:4004");
}

#[test]
fn cli_access_server_sled_engine() {
    cli_access_server("sled", "127.0.0.1:4005");
}
