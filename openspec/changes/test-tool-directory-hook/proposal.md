## Why
Three Gherkin scenarios cannot be verified yet (dry run starts no tool, exit status 2 on partial failure, status through an alias): the real binary always resolves m1ddc and hidapitester from the Homebrew directories first, so a test cannot put fake tools in front of the real ones, and it cannot prove that no tool was started.

## What Changes
- A way for tests (and only tests) to point the tool runner at a directory of fake tools and to record which tools were started.
- The hook must not weaken how tool binaries are trusted (`trusted-tool-binaries`, `harden-tool-execution`): it is compiled only into test builds (a cargo feature or cfg), never read from the environment of a release build.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- None

## Impact
Tool runner and the CLI end-to-end harness (`crates/hostswitchctl/tests/cli.rs`); decision and options belong in design.md (architect, reviewed by the security engineer).

## Out of Scope
Any production override of tool locations (see `trusted-tool-binaries`).
