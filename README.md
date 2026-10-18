# hostswitch

Move your whole desk to another computer with one command: **hostswitch** sets a monitor's input source (DDC/CI) and the Easy-Switch host of a keyboard and mouse together, based on a per-host profile.

**Status: early development (not released).** The command-line tool works on macOS with Apple Silicon; the menu-bar app, remote switching and other platforms are planned. Build from source; there is no installer, signing or App Store build.

## What works today
| Area | State |
|------|-------|
| Switch a host (`switch <host>`, `--dry-run` for the plan only) | works: displays first, then keyboard, then mouse; per-step result; a failed display protects keyboard and mouse |
| Displays | list with current input, standard input ports (`list displays --ports`); set input through `m1ddc` (Apple Silicon) |
| Keyboards and mice | USB receiver (Unifying/Bolt) and Bluetooth LE through `hidapitester`; names and kinds from the hardware |
| Offline devices | remembered, shown as offline, retried once when switching |
| Device aliases | one device reachable over Bluetooth *and* a receiver: `link` declares the second way (needed to switch back from a receiver slot) |
| Configuration | JSON, validated, atomic save; `init` builds a starting config from what it detects; `show` and `status` print it |
| Safety | only channel switching can reach the hardware; no shell; every external call has a timeout |

**Planned** (OpenSpec change proposals in [`openspec/changes/`](openspec/changes)): menu-bar app (`add-tray-app`), state sharing and remote switch between computers (`peer-state-and-remote-switch`), Intel Macs (`intel-display-backend`), native backends without helper programs (`native-backends`), tool version check, and a set of hardening changes.

## Quick start
Requires a recent stable Rust toolchain, `m1ddc` and `hidapitester` (`brew install m1ddc hidapitester`). Bluetooth devices need the macOS *Input Monitoring* permission for your terminal.
```bash
cargo build --release          # binary: target/release/hostswitchctl
hostswitchctl list             # displays and keyboards/mice (read-only)
hostswitchctl init --host "PC=hdmi1" --write   # starting config from the detected hardware
hostswitchctl show             # what each host's profile sets
hostswitchctl status           # compare the profiles with the hardware right now
hostswitchctl switch PC --dry-run   # print the plan
hostswitchctl switch PC             # really switch (display, keyboard, mouse)
```
`switch` changes real hardware. Only the display input and the Easy-Switch host are ever written. If a display step fails, keyboard and mouse stay where they are. Exit status: 0 ok, 1 error, 2 switch partly failed, 64 usage.

A Bluetooth keyboard or mouse can only be switched while it is connected to *this* computer: switch away from here, and switch back from the other computer (or press the Easy-Switch key).

## Configuration
`$XDG_CONFIG_HOME/hostswitch/config.json`, else `~/.config/hostswitch/config.json` (macOS, Linux); `%APPDATA%\Hostswitch\config.json` on Windows; override with `--config <path>`. See [examples/config.example.json](examples/config.example.json):
- `hosts`: one entry per computer (`label` is what you type after `switch`).
- `profiles`: per host the display `input` (DP1 15, DP2 16, HDMI1 17, HDMI2 18, USB-C 27), and `slot` (1 to 3) for `keyboard` and `mouse`; every part is optional.
- `deviceAliases`: other ids of the same device, tried when the first is offline.
- `knownDevices`: filled by `list`/`init`; gives ids readable names.

## Development
```bash
cargo test --workspace         # unit, integration and Gherkin end-to-end tests
cargo clippy --workspace --all-targets && cargo fmt --check
OPENSPEC_TELEMETRY=0 openspec validate --all --strict   # specs and change proposals
python3 scripts/check_traceability.py                    # OpenSpec scenario -> Gherkin -> Rust test
```
- **Workspace:** `hostswitch-core` (domain, config, planner, executor; no OS code, no `unsafe`), `hostswitch-backends` (`m1ddc` and `hidapitester` wrappers, HID++ parsing), `hostswitchctl` (CLI).
- **Specs and tests:** behaviour is specified with [OpenSpec](openspec) (`specs/` = today, `changes/` = proposals); test descriptions are Gherkin `.feature` files in `crates/*/tests/features/`, traced to the specs.
- **Documents:** [requirements](docs/requirements.md) (constraints, risks, ID map) · [architecture decisions](docs/adr/README.md) · research: [DDC](docs/research/ddc.md), [HID++](docs/research/hidpp.md).
- **Agent reports** (test report, security review and other intermediate artefacts) stay local in `.agents/local/`, which git ignores.
- **Working with AI agents:** [AGENTS.md](AGENTS.md) and the role prompts in [.agents/prompts](.agents/prompts) (requirements engineer, architect, developer, tester, security engineer).

## License
Apache-2.0, see [LICENSE](LICENSE). Every source file carries an `SPDX-License-Identifier: Apache-2.0` header.
