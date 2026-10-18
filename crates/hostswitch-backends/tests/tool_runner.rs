// SPDX-License-Identifier: Apache-2.0

//! Integration tests: `ToolRunner` starts real, harmless programs (`sh`, `echo`).

#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use hostswitch_backends::{ToolExecuting, ToolRunner};
use hostswitch_core::HostswitchError;

fn runner(tool: &str, path: &str) -> ToolRunner {
    ToolRunner::new(
        HashMap::from([(tool.to_string(), PathBuf::from(path))]),
        vec![],
    )
}

fn sh(script: &str) -> Vec<String> {
    vec!["-c".into(), script.into()]
}

#[tokio::test]
async fn captures_output_and_exit_code() {
    let r = runner("sh", "/bin/sh")
        .run(
            "sh",
            &sh("echo out; echo err >&2; exit 3"),
            Duration::from_secs(5),
        )
        .await
        .unwrap();
    assert_eq!(
        (r.stdout.trim(), r.stderr.trim(), r.exit_code),
        ("out", "err", 3)
    );
}

#[tokio::test]
async fn timeout_kills_the_process() {
    let started = Instant::now();
    let err = runner("sh", "/bin/sh")
        .run("sh", &sh("sleep 5"), Duration::from_millis(100))
        .await
        .unwrap_err();
    assert!(matches!(err, HostswitchError::TimedOut(_)));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[tokio::test]
async fn large_output_on_both_pipes_does_not_deadlock() {
    let script = "head -c 300000 /dev/zero | tr '\\0' a; head -c 300000 /dev/zero | tr '\\0' b >&2";
    let r = runner("sh", "/bin/sh")
        .run("sh", &sh(script), Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!((r.stdout.len(), r.stderr.len()), (300_000, 300_000));
}

#[tokio::test]
async fn non_utf8_output_is_decoded_lossily() {
    let r = runner("sh", "/bin/sh")
        .run("sh", &sh("printf 'a\\377b'"), Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(r.stdout, "a\u{FFFD}b");
}

#[tokio::test]
async fn arguments_are_not_interpreted_by_a_shell() {
    let args = vec!["$(echo pwned);".to_string()];
    let r = runner("echo", "/bin/echo")
        .run("echo", &args, Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(r.stdout.trim(), "$(echo pwned);");
}

#[tokio::test]
async fn missing_tool_is_backend_unavailable() {
    let runner = ToolRunner::new(HashMap::new(), vec![]);
    let err = runner
        .run(
            "definitely-not-a-real-tool-xyz",
            &[],
            Duration::from_secs(1),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, HostswitchError::BackendUnavailable(_)));
}

#[tokio::test]
async fn override_wins_and_non_executable_override_is_ignored() {
    assert_eq!(
        runner("t", "/bin/echo").resolve("t").await,
        Some(PathBuf::from("/bin/echo"))
    );
    assert_eq!(runner("t", "/nonexistent/t").resolve("t").await, None);
}
