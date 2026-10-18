// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

/// Every failure the core and the backends can report.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HostswitchError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Unsupported configuration version {0}")]
    UnsupportedConfigVersion(i64),
    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("Timed out: {0}")]
    TimedOut(String),
    #[error("Tool failed: {0}")]
    ToolFailed(String),
    #[error("Could not parse tool output: {0}")]
    ParseFailure(String),
    /// The OS refused access to a device (macOS Input Monitoring for Bluetooth HID).
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Unknown host: {0}")]
    UnknownHost(String),
    /// The device did not answer (asleep, on another host, or slot empty). See F-13 / R-6.
    #[error("device offline")]
    DeviceOffline(String),
}

pub type Result<T> = std::result::Result<T, HostswitchError>;
