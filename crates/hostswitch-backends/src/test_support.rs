// SPDX-License-Identifier: Apache-2.0

//! Test doubles for the backends' unit tests (compiled only for tests).

use std::sync::Mutex;

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use hostswitch_core::{HostswitchError, Result};

use crate::tool_runner::{ToolExecuting, ToolResult};

/// Scripted tool executor: replies are consumed in order; every call is recorded.
#[derive(Default)]
pub struct FakeRunner {
    pub available: bool,
    replies: Mutex<std::collections::VecDeque<Result<ToolResult>>>,
    pub calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl FakeRunner {
    pub fn available() -> Self {
        Self {
            available: true,
            ..Self::default()
        }
    }

    pub fn reply(self, stdout: &str, exit_code: i32) -> Self {
        self.push(Ok(ToolResult {
            stdout: stdout.into(),
            stderr: String::new(),
            exit_code,
        }))
    }

    pub fn reply_err(self, stderr: &str, exit_code: i32) -> Self {
        self.push(Ok(ToolResult {
            stdout: String::new(),
            stderr: stderr.into(),
            exit_code,
        }))
    }

    pub fn push(self, r: Result<ToolResult>) -> Self {
        self.replies.lock().unwrap().push_back(r);
        self
    }

    pub fn recorded(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl ToolExecuting for FakeRunner {
    async fn resolve(&self, tool: &str) -> Option<PathBuf> {
        self.available.then(|| PathBuf::from("/fake").join(tool))
    }

    async fn run(
        &self,
        tool: &str,
        arguments: &[String],
        _timeout: Duration,
    ) -> Result<ToolResult> {
        self.calls
            .lock()
            .unwrap()
            .push((tool.to_string(), arguments.to_vec()));
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Err(HostswitchError::ToolFailed("no scripted reply".into())))
    }
}
