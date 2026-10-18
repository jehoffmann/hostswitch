// SPDX-License-Identifier: Apache-2.0

//! Fake backends for tests (enabled with the `testing` feature). They never touch hardware.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use async_trait::async_trait;

use crate::backends::{DisplayBackend, PeripheralBackend};
use crate::domain::{Display, DisplayInput, Peripheral, Slot};
use crate::error::{HostswitchError, Result};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Shared, ordered record of calls across fake backends so tests can assert cross-backend order.
#[derive(Debug, Clone, Default)]
pub struct CallLog(Arc<Mutex<Vec<String>>>);

impl CallLog {
    pub fn record(&self, entry: String) {
        lock(&self.0).push(entry);
    }

    pub fn entries(&self) -> Vec<String> {
        lock(&self.0).clone()
    }
}

#[derive(Debug, Default)]
struct DisplayState {
    inputs: HashMap<String, DisplayInput>,
    /// Number of upcoming set_input calls (per display) that fail before succeeding.
    failures: HashMap<String, u32>,
}

#[derive(Debug, Default)]
pub struct FakeDisplayBackend {
    pub displays: Vec<Display>,
    state: Mutex<DisplayState>,
    /// Async sleep inside set_input (cancellable).
    pub delay: Option<Duration>,
    /// Blocking sleep on a worker thread inside set_input (ignores cancellation, like BUG-1).
    pub block_for: Option<Duration>,
    pub log: CallLog,
}

impl FakeDisplayBackend {
    pub fn new(log: CallLog) -> Self {
        Self {
            log,
            ..Self::default()
        }
    }

    pub fn with_input(self, display: &str, input: DisplayInput) -> Self {
        lock(&self.state).inputs.insert(display.to_string(), input);
        self
    }

    /// Cancellable async sleep inside set_input.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Blocking sleep on a worker thread inside set_input; ignores cancellation (BUG-1).
    pub fn with_block_for(mut self, block: Duration) -> Self {
        self.block_for = Some(block);
        self
    }

    pub fn fail_next(&self, display: &str, count: u32) {
        lock(&self.state)
            .failures
            .insert(display.to_string(), count);
    }

    pub fn input_of(&self, display: &str) -> Option<DisplayInput> {
        lock(&self.state).inputs.get(display).copied()
    }
}

#[async_trait]
impl DisplayBackend for FakeDisplayBackend {
    fn id(&self) -> &str {
        "fake-display"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn list_displays(&self) -> Result<Vec<Display>> {
        Ok(self.displays.clone())
    }

    async fn set_input(&self, input: DisplayInput, display: &str) -> Result<()> {
        self.log.record(format!("display:{display}={}", input.0));
        if let Some(d) = self.delay {
            tokio::time::sleep(d).await;
        }
        if let Some(d) = self.block_for {
            let _ = tokio::task::spawn_blocking(move || std::thread::sleep(d)).await;
        }
        let mut state = lock(&self.state);
        if let Some(n) = state.failures.get_mut(display)
            && *n > 0
        {
            *n -= 1;
            return Err(HostswitchError::ToolFailed("fake display failure".into()));
        }
        state.inputs.insert(display.to_string(), input);
        Ok(())
    }

    async fn current_input(&self, display: &str) -> Result<Option<DisplayInput>> {
        Ok(self.input_of(display))
    }
}

#[derive(Debug, Default)]
struct PeripheralState {
    slots: HashMap<String, Slot>,
    failing: HashSet<String>,
    /// Devices whose switch_host always reports `DeviceOffline`.
    offline: HashSet<String>,
    /// Next N switch_host calls report `DeviceOffline`, then the device "wakes up".
    offline_calls: HashMap<String, u32>,
}

#[derive(Debug, Default)]
pub struct FakePeripheralBackend {
    pub devices: Vec<Peripheral>,
    state: Mutex<PeripheralState>,
    pub log: CallLog,
}

impl FakePeripheralBackend {
    pub fn new(log: CallLog) -> Self {
        Self {
            log,
            ..Self::default()
        }
    }

    pub fn with_slot(self, device: &str, slot: Slot) -> Self {
        lock(&self.state).slots.insert(device.to_string(), slot);
        self
    }

    pub fn set_failing(&self, device: &str) {
        lock(&self.state).failing.insert(device.to_string());
    }

    pub fn set_offline(&self, device: &str) {
        lock(&self.state).offline.insert(device.to_string());
    }

    pub fn set_offline_calls(&self, device: &str, count: u32) {
        lock(&self.state)
            .offline_calls
            .insert(device.to_string(), count);
    }

    pub fn slot_of(&self, device: &str) -> Option<Slot> {
        lock(&self.state).slots.get(device).copied()
    }
}

#[async_trait]
impl PeripheralBackend for FakePeripheralBackend {
    fn id(&self) -> &str {
        "fake-peripheral"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn list_devices(&self) -> Result<Vec<Peripheral>> {
        Ok(self.devices.clone())
    }

    async fn switch_host(&self, slot: Slot, device: &str) -> Result<()> {
        self.log
            .record(format!("peripheral:{device}={}", slot.get()));
        let mut state = lock(&self.state);
        if let Some(n) = state.offline_calls.get_mut(device)
            && *n > 0
        {
            *n -= 1;
            return Err(HostswitchError::DeviceOffline(device.to_string()));
        }
        if state.offline.contains(device) {
            return Err(HostswitchError::DeviceOffline(device.to_string()));
        }
        if state.failing.contains(device) {
            return Err(HostswitchError::ToolFailed(
                "fake peripheral failure".into(),
            ));
        }
        state.slots.insert(device.to_string(), slot);
        Ok(())
    }

    async fn current_host(&self, device: &str) -> Result<Option<Slot>> {
        Ok(self.slot_of(device))
    }
}
