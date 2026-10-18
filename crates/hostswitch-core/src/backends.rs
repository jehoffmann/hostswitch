// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::domain::{Display, DisplayInput, Peripheral, Slot};
use crate::error::Result;

/// Hardware access goes only through these traits (F-06), so tool-backed and native
/// implementations can be swapped per platform.
#[async_trait]
pub trait DisplayBackend: Send + Sync {
    fn id(&self) -> &str;
    async fn is_available(&self) -> bool;
    async fn list_displays(&self) -> Result<Vec<Display>>;
    async fn set_input(&self, input: DisplayInput, display: &str) -> Result<()>;
    async fn current_input(&self, display: &str) -> Result<Option<DisplayInput>>;
}

#[async_trait]
pub trait PeripheralBackend: Send + Sync {
    fn id(&self) -> &str;
    async fn is_available(&self) -> bool;
    async fn list_devices(&self) -> Result<Vec<Peripheral>>;
    async fn switch_host(&self, slot: Slot, device: &str) -> Result<()>;
    async fn current_host(&self, device: &str) -> Result<Option<Slot>>;
}
