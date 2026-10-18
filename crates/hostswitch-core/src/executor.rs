// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use crate::backends::{DisplayBackend, PeripheralBackend};
use crate::error::{HostswitchError, Result};
use crate::planner::SwitchStep;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Failure(String),
    /// Not attempted on purpose (e.g. a display failed, so keyboard/mouse stay put; N-04).
    Skipped(String),
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step: SwitchStep,
    pub outcome: Outcome,
    pub attempts: u32,
}

impl StepResult {
    pub fn is_success(&self) -> bool {
        self.outcome == Outcome::Success
    }
}

#[derive(Debug, Clone, Default)]
pub struct SwitchReport {
    pub results: Vec<StepResult>,
}

impl SwitchReport {
    pub fn is_fully_successful(&self) -> bool {
        self.results.iter().all(StepResult::is_success)
    }

    /// Steps that were attempted and failed (excludes skipped steps).
    pub fn failures(&self) -> impl Iterator<Item = &StepResult> {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, Outcome::Failure(_)))
    }

    pub fn skipped(&self) -> impl Iterator<Item = &StepResult> {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, Outcome::Skipped(_)))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExecutorOptions {
    pub step_timeout: Duration,
    pub max_attempts: u32,
    pub retry_delay: Duration,
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            step_timeout: Duration::from_secs(5),
            max_attempts: 2,
            retry_delay: Duration::from_millis(300),
        }
    }
}

/// Runs steps sequentially. Never fails: every failure is recorded per step (N-04).
///
/// Partial-failure policy (N-04): if any display step fails or times out (after retries), the keyboard and
/// mouse steps are NOT run and are reported as skipped, so the user is never left with peripherals on
/// another host and a display that did not follow. Display steps are independent of each other, and a
/// keyboard failure does not skip the mouse (best-effort, separate step).
/// An offline keyboard/mouse device (`HostswitchError::DeviceOffline`) is retried once after `retry_delay` (it may be
/// waking up), then reported as failed with reason "device offline" (F-13).
pub struct SwitchExecutor {
    display: Option<Arc<dyn DisplayBackend>>,
    peripherals: Option<Arc<dyn PeripheralBackend>>,
    options: ExecutorOptions,
    aliases: BTreeMap<String, Vec<String>>,
}

impl SwitchExecutor {
    pub fn new(
        display: Option<Arc<dyn DisplayBackend>>,
        peripherals: Option<Arc<dyn PeripheralBackend>>,
        options: ExecutorOptions,
    ) -> Self {
        Self {
            display,
            peripherals,
            options,
            aliases: BTreeMap::new(),
        }
    }

    /// Other ids of the same physical device (`AppConfig::device_aliases`): when the primary id is
    /// offline, the aliases are tried in order.
    pub fn with_aliases(mut self, aliases: BTreeMap<String, Vec<String>>) -> Self {
        self.aliases = aliases;
        self
    }

    pub async fn execute(&self, steps: &[SwitchStep]) -> SwitchReport {
        let mut results = Vec::with_capacity(steps.len());
        let mut display_failed = false;
        for step in steps {
            if display_failed && step.is_peripheral() {
                results.push(StepResult {
                    step: step.clone(),
                    outcome: Outcome::Skipped("skipped because a display step failed".into()),
                    attempts: 0,
                });
                continue;
            }
            let result = self.run(step).await;
            if matches!(step, SwitchStep::Display { .. }) && !result.is_success() {
                display_failed = true;
            }
            results.push(result);
        }
        SwitchReport { results }
    }

    async fn run(&self, step: &SwitchStep) -> StepResult {
        let mut last_error = String::from("unknown error");
        let mut limit = self.options.max_attempts.max(1);
        let mut attempt = 0;
        while attempt < limit {
            attempt += 1;
            match with_timeout(
                self.options.step_timeout,
                step.to_string(),
                self.perform(step),
            )
            .await
            {
                Ok(()) => {
                    return StepResult {
                        step: step.clone(),
                        outcome: Outcome::Success,
                        attempts: attempt,
                    };
                }
                Err(error) => {
                    last_error = error.to_string();
                    match error {
                        HostswitchError::BackendUnavailable(_) => break,
                        HostswitchError::DeviceOffline(_) => limit = limit.max(2), // one retry to wake it
                        _ => {}
                    }
                    if attempt < limit {
                        tokio::time::sleep(self.options.retry_delay).await;
                    }
                }
            }
        }
        StepResult {
            step: step.clone(),
            outcome: Outcome::Failure(last_error),
            attempts: attempt,
        }
    }

    async fn perform(&self, step: &SwitchStep) -> Result<()> {
        match step {
            SwitchStep::Display { display_id, input } => {
                let backend = self.display.as_ref().ok_or_else(|| {
                    HostswitchError::BackendUnavailable("no display backend".into())
                })?;
                backend.set_input(*input, display_id).await
            }
            SwitchStep::Keyboard { device_id, slot } | SwitchStep::Mouse { device_id, slot } => {
                let backend = self.peripherals.as_ref().ok_or_else(|| {
                    HostswitchError::BackendUnavailable("no keyboard/mouse backend".into())
                })?;
                // Only "offline" moves on to the next way to reach the device; any other error is real.
                let mut last = HostswitchError::DeviceOffline(device_id.clone());
                let aliases = self.aliases.get(device_id).into_iter().flatten();
                for id in std::iter::once(device_id).chain(aliases) {
                    match backend.switch_host(*slot, id).await {
                        Err(e @ HostswitchError::DeviceOffline(_)) => last = e,
                        other => return other,
                    }
                }
                Err(last)
            }
        }
    }
}

/// Returns the operation's result, or `TimedOut` after `timeout`, whichever happens first.
/// On timeout the operation's future is dropped, never awaited: a backend stuck in blocking code on a
/// worker thread cannot delay the caller (BUG-1); its eventual result is discarded.
pub async fn with_timeout<T>(
    timeout: Duration,
    label: String,
    operation: impl Future<Output = Result<T>>,
) -> Result<T> {
    match tokio::time::timeout(timeout, operation).await {
        Ok(result) => result,
        Err(_) => Err(HostswitchError::TimedOut(label)),
    }
}
