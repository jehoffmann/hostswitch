# Architecture decision records

Each file records one decision: why it was taken, which OpenSpec requirements it satisfies (`openspec/specs/`), and what follows from it. Requirements say *what* the system must do (OpenSpec); ADRs say *why it is built this way*. Format and rules: [ADR-0001](./0001-record-architecture-decisions.md).

| ADR | Decision | Status |
|-----|----------|--------|
| [0001](./0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](./0002-implement-in-rust.md) | Implement hostswitch in Rust | Accepted |
| [0003](./0003-workspace-layout-and-crate-boundaries.md) | Cargo workspace with core, backends and front-ends | Accepted |
| [0004](./0004-hardware-access-behind-backend-interfaces.md) | Hardware access only through backend interfaces | Accepted |
| [0005](./0005-switch-planner-and-executor.md) | Pure switch planner and a never-failing executor with per-step results | Accepted |
| [0006](./0006-configuration-as-versioned-json.md) | Configuration as a versioned, validated JSON file | Accepted |
| [0007](./0007-device-identity-offline-state-and-aliases.md) | Device ids, remembered devices and aliases | Accepted |
| [0008](./0008-external-tools-through-a-tool-runner.md) | External helper programs behind a tool runner with pure parsers | Accepted |
| [0009](./0009-hid-plus-plus-over-receiver-and-bluetooth.md) | HID++ over USB receivers and Bluetooth LE | Accepted |
| [0010](./0010-cli-output-as-pure-render-functions.md) | CLI output produced by pure, tested functions | Accepted |
| [0011](./0011-test-strategy-and-layout.md) | Test strategy: unit, integration and system tests; Gherkin descriptions; traceability | Accepted |
| [0012](./0012-tray-app-with-tray-icon-and-egui.md) | Menu-bar app with tray-icon, muda and egui | Proposed |
| [0013](./0013-peer-state-and-remote-switch-over-mdns.md) | Peer state over mDNS and authenticated remote switching | Proposed |
| [0014](./0014-distribution-build-from-source-apache-2.md) | Distribution: build from source, Apache-2.0 | Accepted |
