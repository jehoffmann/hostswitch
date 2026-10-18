// SPDX-License-Identifier: Apache-2.0

//! hostswitch core: domain types, config, planner and executor. No OS, UI or hardware dependencies (N-07).

pub mod backends;
pub mod bootstrap;
pub mod config;
pub mod domain;
pub mod error;
pub mod executor;
pub mod host_detection;
pub mod planner;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use backends::{DisplayBackend, PeripheralBackend};
pub use config::{AppConfig, ConfigStore};
pub use domain::*;
pub use error::{HostswitchError, Result};
pub use executor::{ExecutorOptions, Outcome, StepResult, SwitchExecutor, SwitchReport};
pub use host_detection::{ObservedState, active_hosts};
pub use planner::{SwitchStep, plan};
