// SPDX-License-Identifier: Apache-2.0

//! Gherkin end-to-end tests for `hostswitchctl` (scenarios tagged `@bdd` in `tests/features/`).
//!
//! The real binary is run against a temporary configuration that only contains obviously fake ids
//! (`ffff:ffff/N`, all-zero UUIDs). Every command used here ends before a hardware tool is started
//! (configuration errors, `--dry-run`, argument errors, `show`, `link`), and `HOME` points to a
//! temporary directory, so no real device or the user's real configuration can be touched.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use cucumber::{World, given, then, when};

static COUNTER: AtomicU32 = AtomicU32::new(0);

const SAMPLE: &str = r#"{
  "version": 2,
  "hosts": [
    { "id": "00000000-0000-4000-8000-000000000001", "label": "Work", "hostname": "mbp.local" },
    { "id": "00000000-0000-4000-8000-000000000002", "label": "Home", "hostname": "" },
    { "id": "00000000-0000-4000-8000-000000000003", "label": "Bare", "hostname": "" },
    { "id": "00000000-0000-4000-8000-000000000004", "label": "Empty", "hostname": "" }
  ],
  "profiles": [
    {
      "hostID": "00000000-0000-4000-8000-000000000001",
      "displays": [ { "displayID": "00000000-0000-4000-8000-0000000000d1", "input": 15 } ],
      "keyboard": { "deviceID": "ffff:ffff/1", "slot": 1 },
      "mouse": { "deviceID": "ffff:ffff/2", "slot": 1 }
    },
    {
      "hostID": "00000000-0000-4000-8000-000000000002",
      "displays": [ { "displayID": "00000000-0000-4000-8000-0000000000d1", "input": 17 } ],
      "keyboard": { "deviceID": "ffff:ffff/1", "slot": 2 },
      "mouse": { "deviceID": "ffff:ffff/2", "slot": 2 }
    },
    { "hostID": "00000000-0000-4000-8000-000000000004" }
  ],
  "knownDevices": [
    { "id": "ffff:ffff/1", "name": "Test Keyboard", "kind": "keyboard", "transport": "unifying" },
    { "id": "ffff:ffff/2", "name": "Test Mouse", "kind": "mouse", "transport": "unifying" }
  ]
}"#;

const EMPTY: &str = r#"{ "version": 2, "hosts": [], "profiles": [] }"#;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct CliWorld {
    dir: PathBuf,
    config: PathBuf,
    /// The configuration file as written by the `Given` step (None = no file).
    snapshot: Option<Vec<u8>>,
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

impl CliWorld {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "hostswitch-bdd-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        let config = dir.join("config.json");
        Self {
            dir,
            config,
            snapshot: None,
            status: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    fn write_config(&mut self, text: &str) {
        fs::write(&self.config, text).unwrap();
        self.snapshot = Some(fs::read(&self.config).unwrap());
    }
}

impl Drop for CliWorld {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// Splits `a "b c" d` into arguments; double quotes group words.
fn split_args(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut started = false;
    for c in text.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                started = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if started {
                    args.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            c => {
                current.push(c);
                started = true;
            }
        }
    }
    if started {
        args.push(current);
    }
    args
}

#[given("the sample configuration")]
fn sample_configuration(world: &mut CliWorld) {
    world.write_config(SAMPLE);
}

#[given("an empty configuration")]
fn empty_configuration(world: &mut CliWorld) {
    world.write_config(EMPTY);
}

#[given("no configuration file")]
fn no_configuration_file(world: &mut CliWorld) {
    let _ = fs::remove_file(&world.config);
    world.snapshot = None;
}

#[when(expr = "I run {string}")]
fn run(world: &mut CliWorld, command: String) {
    let mut args = split_args(&command);
    assert_eq!(
        args.first().map(String::as_str),
        Some("hostswitchctl"),
        "{command}"
    );
    args.remove(0);
    if !args.iter().any(|a| a == "--config") {
        args.splice(
            0..0,
            ["--config".to_string(), world.config.display().to_string()],
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_hostswitchctl"))
        .args(&args)
        .env("HOME", &world.dir)
        .env("XDG_CONFIG_HOME", world.dir.join("xdg"))
        .env("OPENSPEC_TELEMETRY", "0")
        .output()
        .unwrap();
    world.status = output.status.code();
    world.stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    world.stderr = String::from_utf8_lossy(&output.stderr).into_owned();
}

#[then(expr = "the exit status is {int}")]
fn exit_status(world: &mut CliWorld, expected: i32) {
    assert_eq!(
        world.status,
        Some(expected),
        "stdout:\n{}\nstderr:\n{}",
        world.stdout,
        world.stderr
    );
}

#[then(expr = "the output contains {string}")]
fn output_contains(world: &mut CliWorld, text: String) {
    assert!(
        world.stdout.contains(&text),
        "missing {text:?} in:\n{}",
        world.stdout
    );
}

#[then(expr = "the error output contains {string}")]
fn error_output_contains(world: &mut CliWorld, text: String) {
    assert!(
        world.stderr.contains(&text),
        "missing {text:?} in:\n{}",
        world.stderr
    );
}

/// Step results are only printed by a switch that really ran steps.
#[then("no step result is printed")]
fn no_step_result(world: &mut CliWorld) {
    for line in world.stdout.lines() {
        let line = line.trim_start();
        assert!(
            !(line.starts_with("OK ") || line.starts_with("FAIL ") || line.starts_with("SKIP ")),
            "a step ran: {line}"
        );
    }
}

#[then("the configuration file is unchanged")]
fn configuration_unchanged(world: &mut CliWorld) {
    let now = fs::read(&world.config).ok();
    assert_eq!(now, world.snapshot, "configuration file changed");
}

#[then(expr = "the configuration file contains {string}")]
fn configuration_contains(world: &mut CliWorld, text: String) {
    let content = fs::read_to_string(&world.config).unwrap();
    assert!(content.contains(&text), "missing {text:?} in:\n{content}");
}

#[then("no configuration file exists")]
fn no_configuration_file_exists(world: &mut CliWorld) {
    assert!(!world.config.exists(), "{} exists", world.config.display());
}

#[tokio::main]
async fn main() {
    CliWorld::cucumber()
        .fail_on_skipped()
        .filter_run("tests/features", |_, _, scenario| {
            scenario.tags.iter().any(|tag| tag == "bdd")
        })
        .await;
}
