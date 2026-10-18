// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use hostswitch_core::{HostswitchError, Result};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Seam so backends can be tested without spawning processes.
#[async_trait]
pub trait ToolExecuting: Send + Sync {
    async fn resolve(&self, tool: &str) -> Option<PathBuf>;
    async fn run(&self, tool: &str, arguments: &[String], timeout: Duration) -> Result<ToolResult>;
}

/// Resolves external binaries (explicit override -> Homebrew paths -> PATH) and runs them with a timeout.
#[derive(Debug, Clone)]
pub struct ToolRunner {
    overrides: HashMap<String, PathBuf>,
    search_paths: Vec<PathBuf>,
}

impl Default for ToolRunner {
    fn default() -> Self {
        Self::new(HashMap::new(), Self::default_search_paths())
    }
}

impl ToolRunner {
    pub fn new(overrides: HashMap<String, PathBuf>, search_paths: Vec<PathBuf>) -> Self {
        Self {
            overrides,
            search_paths,
        }
    }

    pub fn default_search_paths() -> Vec<PathBuf> {
        ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"]
            .iter()
            .map(PathBuf::from)
            .collect()
    }

    fn resolve_sync(&self, tool: &str) -> Option<PathBuf> {
        if let Some(path) = self.overrides.get(tool)
            && is_executable(path)
        {
            return Some(path.clone());
        }
        let path_env = std::env::var_os("PATH").unwrap_or_default();
        self.search_paths
            .iter()
            .cloned()
            .chain(std::env::split_paths(&path_env))
            .map(|dir| dir.join(tool))
            .find(|candidate| is_executable(candidate))
    }
}

fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

#[async_trait]
impl ToolExecuting for ToolRunner {
    async fn resolve(&self, tool: &str) -> Option<PathBuf> {
        self.resolve_sync(tool)
    }

    async fn run(&self, tool: &str, arguments: &[String], timeout: Duration) -> Result<ToolResult> {
        let executable = self
            .resolve_sync(tool)
            .ok_or_else(|| HostswitchError::BackendUnavailable(format!("{tool} not found")))?;
        run_process(&executable, arguments, timeout, tool).await
    }
}

/// Arguments are passed as separate elements (never through a shell). The child is killed when the
/// timeout drops the future (`kill_on_drop`). Output is decoded lossily: device names may be non-UTF8.
pub(crate) async fn run_process(
    executable: &std::path::Path,
    arguments: &[String],
    timeout: Duration,
    label: &str,
) -> Result<ToolResult> {
    let mut command = Command::new(executable);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let output = match tokio::time::timeout(timeout, command.output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(HostswitchError::ToolFailed(format!("{label}: {e}"))),
        Err(_) => {
            return Err(HostswitchError::TimedOut(format!(
                "{label} {}",
                arguments.join(" ")
            )));
        }
    };
    Ok(ToolResult {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}
