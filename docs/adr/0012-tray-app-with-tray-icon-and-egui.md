# ADR-0012: Menu-bar app with tray-icon, muda and egui

- Status: Proposed
- Date: 2026-10-18

## Context
F-01 and F-02 ask for a menu-bar item and a settings window. The author wants to practise Rust, so as little non-Rust UI code as possible.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- change:add-tray-app/F-01 — menu lists hosts and switches
- change:add-tray-app/F-02 — edit hosts and profiles
- change:add-tray-app/F-09 — which host owns which device
- change:add-tray-app/F-12 — hotkey per host

## Decision
A new crate `hostswitch-app`: native tray menu with `tray-icon` and `muda`, a settings window with `egui`/`eframe`. A UI-free view model (configuration editing, refresh, switch summary) is tested with the core fakes; its wording is that of `render.rs`. The app reuses core and backends and adds no hardware command. A spike for tray plus egui event-loop integration on macOS comes first.

## Consequences
- Pure Rust and cross-platform; a native menu on macOS.
- The event-loop integration of tray and egui is unproven on macOS; the spike decides.
- The app holds the Input Monitoring permission, which makes the trust in helper programs urgent (`trusted-tool-binaries`).

## Alternatives considered
- Tauri 2 (web UI): quick to build and documented, but little Rust to practise.
- A SwiftUI shell over a Rust core (UniFFI): best native look, not cross-platform.
