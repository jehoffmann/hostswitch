## Why
F-01, F-02, F-09 and F-12 are still open: there is no menu-bar app, no settings window and no hotkey. Today everything needs the command line.

## What Changes
- A menu-bar (tray) app lists the configured hosts, marks the active one and switches on selection.
- A settings window creates, edits and deletes hosts and profiles with pick-lists of detected displays and devices.
- A status line shows which host currently owns which device; a global hotkey per host (later iteration).

## Capabilities
### New Capabilities
- `tray-app`

### Modified Capabilities
- None

## Impact
New crate `hostswitch-app` (tray-icon, muda, egui per ADR-0012). Reuses core and backends; no new hardware command.

## Out of Scope
Remote switching and peer state (change `peer-state-and-remote-switch`).
