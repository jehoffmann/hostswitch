// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hostswitch_core::{Display, DisplayBackend, DisplayInput, HostswitchError, Result};

use crate::tool_runner::{ToolExecuting, ToolResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedDisplay {
    pub index: u32,
    pub name: String,
    pub uuid: String,
}

/// Lines look like `[1] MSI MPG341CQR (1C95A13D-CA9A-4160-A4BD-E0DFA6B99D52)`.
/// Detailed output ("- Key: value" lines) is tolerated: those lines are skipped.
pub fn parse_display_list(output: &str) -> Vec<ListedDisplay> {
    output.lines().filter_map(parse_display_line).collect()
}

fn parse_display_line(line: &str) -> Option<ListedDisplay> {
    let text = line.trim();
    let rest = text.strip_prefix('[')?;
    let close = rest.find(']')?;
    let index = rest[..close].parse().ok()?;
    let after = &rest[close + 1..];
    let after = after.strip_suffix(')')?;
    let open = after.rfind('(')?;
    let uuid = after[open + 1..].to_string();
    let name = after[..open].trim().to_string();
    (!uuid.is_empty() && !name.is_empty()).then_some(ListedDisplay { index, name, uuid })
}

/// `get input` prints a bare integer. Empty output is a transient failure, never "no input" (docs/research/ddc.md).
pub fn parse_input(output: &str) -> Option<DisplayInput> {
    output.trim().parse::<u16>().ok().map(DisplayInput)
}

/// Command construction + parsing only; all process handling lives in `ToolExecuting`.
pub struct M1ddcBackend {
    runner: Arc<dyn ToolExecuting>,
    timeout: Duration,
    read_attempts: u32,
    retry_delay: Duration,
}

impl M1ddcBackend {
    pub fn new(runner: Arc<dyn ToolExecuting>) -> Self {
        Self {
            runner,
            timeout: Duration::from_secs(4),
            read_attempts: 3,
            retry_delay: Duration::from_millis(300),
        }
    }

    pub fn with_retry(mut self, read_attempts: u32, retry_delay: Duration) -> Self {
        self.read_attempts = read_attempts.max(1);
        self.retry_delay = retry_delay;
        self
    }

    fn list_arguments() -> Vec<String> {
        vec!["display".into(), "list".into()]
    }

    fn get_input_arguments(display: &str) -> Vec<String> {
        ["display", display, "get", "input"]
            .map(String::from)
            .to_vec()
    }

    fn set_input_arguments(display: &str, input: DisplayInput) -> Vec<String> {
        ["display", display, "set", "input", &input.0.to_string()]
            .map(String::from)
            .to_vec()
    }

    fn failure(result: &ToolResult) -> HostswitchError {
        let detail = [&result.stderr, &result.stdout]
            .into_iter()
            .map(|s| s.trim())
            .find(|s| !s.is_empty())
            .unwrap_or("empty output");
        HostswitchError::ToolFailed(format!("m1ddc exit {}: {detail}", result.exit_code))
    }
}

#[async_trait]
impl DisplayBackend for M1ddcBackend {
    fn id(&self) -> &str {
        "m1ddc"
    }

    async fn is_available(&self) -> bool {
        cfg!(all(target_os = "macos", target_arch = "aarch64"))
            && self.runner.resolve("m1ddc").await.is_some()
    }

    async fn list_displays(&self) -> Result<Vec<Display>> {
        let result = self
            .runner
            .run("m1ddc", &Self::list_arguments(), self.timeout)
            .await?;
        if result.exit_code != 0 {
            return Err(Self::failure(&result));
        }
        // Stable UUID is the id, so config survives index reordering (docs/research/ddc.md).
        Ok(parse_display_list(&result.stdout)
            .into_iter()
            .map(|d| Display {
                id: d.uuid,
                name: d.name,
                backend_id: self.id().to_string(),
            })
            .collect())
    }

    async fn set_input(&self, input: DisplayInput, display: &str) -> Result<()> {
        let result = self
            .runner
            .run(
                "m1ddc",
                &Self::set_input_arguments(display, input),
                self.timeout,
            )
            .await?;
        if result.exit_code != 0 {
            return Err(Self::failure(&result));
        }
        Ok(())
    }

    async fn current_input(&self, display: &str) -> Result<Option<DisplayInput>> {
        let mut last = HostswitchError::ToolFailed("m1ddc get input: no output".into());
        for attempt in 1..=self.read_attempts {
            match self
                .runner
                .run("m1ddc", &Self::get_input_arguments(display), self.timeout)
                .await
            {
                Ok(result) => {
                    if result.exit_code == 0
                        && let Some(input) = parse_input(&result.stdout)
                    {
                        return Ok(Some(input));
                    }
                    last = Self::failure(&result);
                }
                Err(e @ HostswitchError::BackendUnavailable(_)) => return Err(e),
                Err(e) => last = e,
            }
            if attempt < self.read_attempts {
                tokio::time::sleep(self.retry_delay).await;
            }
        }
        Err(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::FakeRunner;

    const LIST: &str = include_str!("../tests/fixtures/m1ddc-display-list.txt");
    const DETAILED: &str = include_str!("../tests/fixtures/m1ddc-display-list-detailed.txt");
    const GET: &str = include_str!("../tests/fixtures/m1ddc-get-input.txt");
    const ERR: &str = include_str!("../tests/fixtures/m1ddc-error-no-display.txt");
    const UUID: &str = "1C95A13D-CA9A-4160-A4BD-E0DFA6B99D52";

    fn backend(runner: FakeRunner) -> (M1ddcBackend, Arc<FakeRunner>) {
        let runner = Arc::new(runner);
        (
            M1ddcBackend::new(runner.clone()).with_retry(3, Duration::from_millis(1)),
            runner,
        )
    }

    #[test]
    fn parses_recorded_list_and_detailed_list() {
        let expected = vec![ListedDisplay {
            index: 1,
            name: "MSI MPG341CQR".into(),
            uuid: UUID.into(),
        }];
        assert_eq!(parse_display_list(LIST), expected);
        assert_eq!(
            parse_display_list(DETAILED),
            expected,
            "detail lines are skipped"
        );
    }

    #[test]
    fn list_parser_tolerates_garbage() {
        for garbage in [
            "",
            "\n\n",
            "[x] A (b)",
            "[1] (uuid)",
            "[1] Name ()",
            "[1] Name",
            "random text (with parens)",
            ERR,
        ] {
            assert!(parse_display_list(garbage).is_empty(), "{garbage:?}");
        }
        let two = "[1] A (u1)\r\n[2] B C (u2)\r\n";
        assert_eq!(parse_display_list(two).len(), 2);
        assert_eq!(parse_display_list(two)[1].name, "B C");
    }

    #[test]
    fn input_parser_bounds() {
        assert_eq!(parse_input(GET), Some(DisplayInput(15)));
        assert_eq!(parse_input(" 27\n"), Some(DisplayInput(27)));
        assert_eq!(parse_input("65535"), Some(DisplayInput(65535)));
        for bad in ["", "  ", "65536", "-1", "abc", "15 16"] {
            assert_eq!(parse_input(bad), None, "{bad:?}");
        }
    }

    #[tokio::test]
    async fn lists_displays_using_uuid_as_id() {
        let (b, runner) = backend(FakeRunner::available().reply(LIST, 0));
        let displays = b.list_displays().await.unwrap();
        assert_eq!(displays[0].id, UUID);
        assert_eq!(runner.recorded()[0].1, ["display", "list"]);
    }

    #[tokio::test]
    async fn set_input_builds_the_expected_command() {
        let (b, runner) = backend(FakeRunner::available().reply("", 0));
        b.set_input(DisplayInput(17), UUID).await.unwrap();
        assert_eq!(
            runner.recorded()[0].1,
            ["display", UUID, "set", "input", "17"]
        );
    }

    #[tokio::test]
    async fn set_input_reports_tool_error_text() {
        let (b, _) = backend(FakeRunner::available().reply_err(ERR, 1));
        let err = b.set_input(DisplayInput(17), "9").await.unwrap_err();
        assert!(err.to_string().contains("does not exist"), "{err}");
    }

    #[tokio::test]
    async fn empty_first_read_is_retried_and_never_means_no_input() {
        let (b, runner) = backend(FakeRunner::available().reply("", 1).reply(GET, 0));
        assert_eq!(b.current_input(UUID).await.unwrap(), Some(DisplayInput(15)));
        assert_eq!(runner.recorded().len(), 2);

        let (b, runner) = backend(
            FakeRunner::available()
                .reply("", 0)
                .reply("", 0)
                .reply("", 0),
        );
        assert!(b.current_input(UUID).await.is_err());
        assert_eq!(runner.recorded().len(), 3);
    }

    #[tokio::test]
    async fn unavailable_tool_stops_retrying() {
        let (b, runner) = backend(
            FakeRunner::available().push(Err(HostswitchError::BackendUnavailable("x".into()))),
        );
        assert!(matches!(
            b.current_input(UUID).await,
            Err(HostswitchError::BackendUnavailable(_))
        ));
        assert_eq!(runner.recorded().len(), 1);
    }

    #[tokio::test]
    async fn availability_requires_the_binary() {
        let (b, _) = backend(FakeRunner::default());
        assert!(!b.is_available().await);
    }
}
