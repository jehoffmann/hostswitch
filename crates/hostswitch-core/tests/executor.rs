// SPDX-License-Identifier: Apache-2.0

//! Integration tests: executor behaviour on the public API with the fake backends.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hostswitch_core::testing::*;
use hostswitch_core::*;

fn slot(n: u8) -> Slot {
    Slot::new(n).unwrap()
}

fn fast() -> ExecutorOptions {
    ExecutorOptions {
        step_timeout: Duration::from_millis(200),
        max_attempts: 2,
        retry_delay: Duration::from_millis(1),
    }
}

fn profile() -> Profile {
    let mut p = Profile::new(uuid::Uuid::new_v4());
    p.displays.push(DisplayAssignment {
        display_id: "d1".into(),
        input: DisplayInput(17),
    });
    p.keyboard = Some(DeviceAssignment {
        device_id: "kb".into(),
        slot: slot(2),
    });
    p.mouse = Some(DeviceAssignment {
        device_id: "ms".into(),
        slot: slot(2),
    });
    p
}

struct Rig {
    display: Arc<FakeDisplayBackend>,
    peripherals: Arc<FakePeripheralBackend>,
    log: CallLog,
}

impl Rig {
    fn new() -> Self {
        let log = CallLog::default();
        Self {
            display: Arc::new(FakeDisplayBackend::new(log.clone())),
            peripherals: Arc::new(FakePeripheralBackend::new(log.clone())),
            log,
        }
    }

    fn executor(&self) -> SwitchExecutor {
        SwitchExecutor::new(
            Some(self.display.clone()),
            Some(self.peripherals.clone()),
            fast(),
        )
    }
}

#[tokio::test]
async fn happy_path_runs_in_order_across_backends() {
    let rig = Rig::new();
    let report = rig.executor().execute(&plan(&profile())).await;
    assert!(report.is_fully_successful());
    assert_eq!(
        rig.log.entries(),
        ["display:d1=17", "peripheral:kb=2", "peripheral:ms=2"]
    );
    assert_eq!(rig.display.input_of("d1"), Some(DisplayInput(17)));
    assert!(report.results.iter().all(|r| r.attempts == 1));
}

#[tokio::test]
async fn transient_failure_is_retried() {
    let rig = Rig::new();
    rig.display.fail_next("d1", 1);
    let report = rig.executor().execute(&plan(&profile())).await;
    assert!(report.is_fully_successful());
    assert_eq!(report.results[0].attempts, 2);
}

#[tokio::test]
async fn failed_display_skips_keyboard_and_mouse() {
    let rig = Rig::new();
    rig.display.fail_next("d1", 10);
    let report = rig.executor().execute(&plan(&profile())).await;
    assert!(matches!(report.results[0].outcome, Outcome::Failure(_)));
    assert!(matches!(report.results[1].outcome, Outcome::Skipped(_)));
    assert!(matches!(report.results[2].outcome, Outcome::Skipped(_)));
    assert_eq!(report.skipped().count(), 2);
    assert_eq!(report.failures().count(), 1);
    assert_eq!(
        rig.log.entries(),
        ["display:d1=17", "display:d1=17"],
        "no peripheral call was made"
    );
}

#[tokio::test]
async fn other_displays_still_run_after_a_display_failure() {
    let rig = Rig::new();
    rig.display.fail_next("d1", 10);
    let mut p = profile();
    p.displays.push(DisplayAssignment {
        display_id: "d2".into(),
        input: DisplayInput(15),
    });
    let report = rig.executor().execute(&plan(&p)).await;
    assert!(matches!(report.results[0].outcome, Outcome::Failure(_)));
    assert!(report.results[1].is_success());
    assert!(matches!(report.results[2].outcome, Outcome::Skipped(_)));
}

#[tokio::test]
async fn keyboard_failure_does_not_skip_mouse() {
    let rig = Rig::new();
    rig.peripherals.set_failing("kb");
    let report = rig.executor().execute(&plan(&profile())).await;
    assert!(report.results[0].is_success());
    assert!(matches!(report.results[1].outcome, Outcome::Failure(_)));
    assert!(report.results[2].is_success());
    assert_eq!(rig.peripherals.slot_of("ms"), Some(slot(2)));
}

#[tokio::test]
async fn missing_backend_fails_without_retry() {
    let report = SwitchExecutor::new(None, None, fast())
        .execute(&plan(&profile()))
        .await;
    assert_eq!(report.results[0].attempts, 1);
    assert_eq!(
        report.results[0].outcome,
        Outcome::Failure("Backend unavailable: no display backend".into())
    );
    assert!(matches!(report.results[1].outcome, Outcome::Skipped(_)));

    let mut kb_only = Profile::new(uuid::Uuid::new_v4());
    kb_only.keyboard = Some(DeviceAssignment {
        device_id: "kb".into(),
        slot: slot(1),
    });
    let report = SwitchExecutor::new(None, None, fast())
        .execute(&plan(&kb_only))
        .await;
    assert_eq!(
        report.results[0].outcome,
        Outcome::Failure("Backend unavailable: no keyboard/mouse backend".into())
    );
}

#[tokio::test]
async fn offline_device_is_retried_once_then_reported_offline() {
    let rig = Rig::new();
    rig.peripherals.set_offline("kb");
    let opts = ExecutorOptions {
        max_attempts: 1,
        ..fast()
    };
    let ex = SwitchExecutor::new(
        Some(rig.display.clone()),
        Some(rig.peripherals.clone()),
        opts,
    );
    let report = ex.execute(&plan(&profile())).await;
    let kb = &report.results[1];
    assert_eq!(kb.outcome, Outcome::Failure("device offline".into()));
    assert_eq!(kb.attempts, 2, "one retry even when max_attempts is 1");
    assert!(report.results[2].is_success(), "mouse unaffected");
}

#[tokio::test]
async fn offline_device_waking_on_retry_succeeds() {
    let rig = Rig::new();
    rig.peripherals.set_offline_calls("kb", 1);
    let report = rig.executor().execute(&plan(&profile())).await;
    assert!(report.is_fully_successful());
    assert_eq!(report.results[1].attempts, 2);
}

#[tokio::test(start_paused = true)]
async fn cancellable_slow_step_times_out_and_skips_peripherals() {
    let log = CallLog::default();
    let display =
        Arc::new(FakeDisplayBackend::new(log.clone()).with_delay(Duration::from_secs(60)));
    let peripherals = Arc::new(FakePeripheralBackend::new(log));
    let ex = SwitchExecutor::new(
        Some(display),
        Some(peripherals),
        ExecutorOptions {
            max_attempts: 1,
            ..fast()
        },
    );
    let report = ex.execute(&plan(&profile())).await;
    assert_eq!(
        report.results[0].outcome,
        Outcome::Failure("Timed out: display d1 -> HDMI1 (17)".into())
    );
    assert!(matches!(report.results[1].outcome, Outcome::Skipped(_)));
}

/// BUG-1: a backend blocked on a worker thread that ignores cancellation must not hold the caller.
#[tokio::test]
async fn blocking_backend_cannot_delay_the_caller_past_the_timeout() {
    let log = CallLog::default();
    let display =
        Arc::new(FakeDisplayBackend::new(log).with_block_for(Duration::from_millis(1500)));
    let opts = ExecutorOptions {
        step_timeout: Duration::from_millis(100),
        max_attempts: 1,
        ..fast()
    };
    let ex = SwitchExecutor::new(Some(display), None, opts);
    let started = Instant::now();
    let report = ex
        .execute(&[SwitchStep::Display {
            display_id: "d".into(),
            input: DisplayInput(1),
        }])
        .await;
    assert!(
        started.elapsed() < Duration::from_millis(1000),
        "took {:?}",
        started.elapsed()
    );
    assert!(matches!(report.results[0].outcome, Outcome::Failure(_)));
}

#[tokio::test]
async fn empty_plan_is_an_empty_successful_report() {
    let report = Rig::new().executor().execute(&[]).await;
    assert!(report.results.is_empty());
    assert!(report.is_fully_successful());
}

fn aliased(rig: &Rig) -> SwitchExecutor {
    SwitchExecutor::new(
        Some(rig.display.clone()),
        Some(rig.peripherals.clone()),
        fast(),
    )
    .with_aliases(BTreeMap::from([(
        "kb".to_string(),
        vec!["kb-usb".to_string(), "kb-3".to_string()],
    )]))
}

fn keyboard_only() -> Vec<SwitchStep> {
    vec![SwitchStep::Keyboard {
        device_id: "kb".into(),
        slot: slot(3),
    }]
}

#[tokio::test]
async fn offline_primary_falls_back_to_the_alias() {
    let rig = Rig::new();
    rig.peripherals.set_offline("kb");
    let report = aliased(&rig).execute(&keyboard_only()).await;
    assert!(report.is_fully_successful(), "{report:?}");
    assert_eq!(
        rig.log.entries(),
        ["peripheral:kb=3", "peripheral:kb-usb=3"]
    );
    assert_eq!(rig.peripherals.slot_of("kb-usb"), Some(slot(3)));
}

#[tokio::test]
async fn working_primary_never_touches_aliases() {
    let rig = Rig::new();
    let report = aliased(&rig).execute(&keyboard_only()).await;
    assert!(report.is_fully_successful());
    assert_eq!(rig.log.entries(), ["peripheral:kb=3"]);
}

#[tokio::test]
async fn real_errors_are_not_masked_by_aliases() {
    let rig = Rig::new();
    rig.peripherals.set_failing("kb");
    let report = aliased(&rig).execute(&keyboard_only()).await;
    assert!(
        matches!(report.results[0].outcome, Outcome::Failure(ref m) if m.contains("fake peripheral failure"))
    );
    assert_eq!(
        rig.log.entries().len(),
        2,
        "two attempts on the primary only"
    );
    assert!(rig.log.entries().iter().all(|e| e == "peripheral:kb=3"));
}

#[tokio::test]
async fn all_ways_offline_is_reported_as_device_offline() {
    let rig = Rig::new();
    for id in ["kb", "kb-usb", "kb-3"] {
        rig.peripherals.set_offline(id);
    }
    let report = aliased(&rig).execute(&keyboard_only()).await;
    assert_eq!(
        report.results[0].outcome,
        Outcome::Failure("device offline".into())
    );
    assert_eq!(
        report.results[0].attempts, 2,
        "retried once, each pass tries every id"
    );
    assert_eq!(rig.log.entries().len(), 6);
}
