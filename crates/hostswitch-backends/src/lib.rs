// SPDX-License-Identifier: Apache-2.0

//! Wrappers for external tools (`m1ddc`, `hidapitester`) plus the pure parsers behind them (N-03).

pub mod hid_enum;
pub mod hidapitester;
pub mod hidpp;
pub mod m1ddc;
pub mod registry;
#[cfg(test)]
mod test_support;
pub mod tool_runner;

pub use hidapitester::HidapiTesterPeripheralBackend;
pub use m1ddc::M1ddcBackend;
pub use registry::BackendRegistry;
pub use tool_runner::{ToolExecuting, ToolResult, ToolRunner};
