// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use hostswitch_core::{DisplayBackend, PeripheralBackend};

use crate::hidapitester::HidapiTesterPeripheralBackend;
use crate::m1ddc::M1ddcBackend;
use crate::tool_runner::{ToolExecuting, ToolRunner};

/// Picks the best available backend per capability at runtime.
#[derive(Clone)]
pub struct BackendRegistry {
    display_candidates: Vec<Arc<dyn DisplayBackend>>,
    peripheral_candidates: Vec<Arc<dyn PeripheralBackend>>,
}

impl BackendRegistry {
    pub fn new(
        display_candidates: Vec<Arc<dyn DisplayBackend>>,
        peripheral_candidates: Vec<Arc<dyn PeripheralBackend>>,
    ) -> Self {
        Self {
            display_candidates,
            peripheral_candidates,
        }
    }

    /// Apple Silicon -> m1ddc. TODO: ddcctl backend for Intel (R-4); native hidapi/DDC backends per OS.
    pub fn with_default_backends(runner: Arc<dyn ToolExecuting>) -> Self {
        Self::new(
            vec![Arc::new(M1ddcBackend::new(runner.clone()))],
            vec![Arc::new(HidapiTesterPeripheralBackend::new(runner))],
        )
    }

    pub fn default_backends() -> Self {
        Self::with_default_backends(Arc::new(ToolRunner::default()))
    }

    pub async fn display(&self) -> Option<Arc<dyn DisplayBackend>> {
        for candidate in &self.display_candidates {
            if candidate.is_available().await {
                return Some(candidate.clone());
            }
        }
        None
    }

    pub async fn peripherals(&self) -> Option<Arc<dyn PeripheralBackend>> {
        for candidate in &self.peripheral_candidates {
            if candidate.is_available().await {
                return Some(candidate.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hostswitch_core::testing::{CallLog, FakeDisplayBackend, FakePeripheralBackend};

    #[tokio::test]
    async fn picks_the_first_available_candidate() {
        let log = CallLog::default();
        let r = BackendRegistry::new(
            vec![Arc::new(FakeDisplayBackend::new(log.clone()))],
            vec![Arc::new(FakePeripheralBackend::new(log))],
        );
        assert_eq!(
            r.display().await.map(|b| b.id().to_string()),
            Some("fake-display".into())
        );
        assert_eq!(
            r.peripherals().await.map(|b| b.id().to_string()),
            Some("fake-peripheral".into())
        );
        assert!(
            BackendRegistry::new(vec![], vec![])
                .display()
                .await
                .is_none()
        );
    }
}
