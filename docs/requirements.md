# Requirements — hostswitch

hostswitch moves a desk to another computer: it sets a display's input (DDC) and the Easy-Switch host of a keyboard and mouse together.

**Behaviour is specified in OpenSpec** — `openspec/specs/<capability>/spec.md` is the source of truth for what the system does (requirements with scenarios); proposed work lives in `openspec/changes/<name>/`. This file only holds what OpenSpec specs deliberately do not: glossary, project constraints (non-behavioural), risks, and the map from the legacy requirement IDs to the specs. IDs are stable; code and tests reference them.

## 1. Glossary
- **Host** — a computer, identified by a stable id, a label and a hostname.
- **Profile** — what the desk looks like when a host is active: a list of (display id, input), an optional keyboard `(device id, slot)` and an optional mouse `(device id, slot)`.
- **Display input** — DDC/CI VCP 0x60 value (15 = DP1, 17 = HDMI1, 27 = USB-C, ...).
- **Slot** — Easy-Switch host index 1..3 (HID++ CHANGE_HOST).
- **Device id** — `vid:pid/index`: a receiver slot (1..6) or a directly connected Bluetooth device (255). **Alias** — another id of the same physical device.

## 2. Where each requirement lives
| Legacy ID | Spec (`openspec/specs/…` or change) |
|-----------|-------------------------------------|
| F-01, F-01a..c, F-03, F-04, N-04a..c | `switching` |
| F-02a..f | `config` |
| F-05, F-05a, F-05b, R-3a, R-5, N-03a | `discovery` |
| F-13, F-13a..c | `offline-devices` |
| R-2a..c | `device-aliases` |
| F-07, F-07a..f | `cli` |
| S-01..S-05, N-04 safety rules | `safety` |
| N-03, N-03b, N-03c | `tools` |
| F-01/F-02/F-09/F-12 (UI, hotkey) | change `add-tray-app` (open) |
| F-08, F-10, F-11 | change `peer-state-and-remote-switch` (open) |
| N-01 (Intel), R-4 | change `intel-display-backend` (open) |
| N-03d (versions) | change `tool-version-check` (open) |
| SEC-1..SEC-14, S-06..S-08 | hardening changes: `fix-config-clobber`, `harden-config-storage`, `validate-identifiers`, `harden-tool-execution`, `write-timeout-semantics`, `verify-device-identity`, `sanitize-hardware-text`, `trusted-tool-binaries`, `dependency-hygiene`, `hardware-command-allowlist` |
| F-06, F-06a | ADR-0004 (docs/adr): hardware only behind backend interfaces |

## 3. Constraints (non-behavioural)
| ID | Constraint |
|----|------------|
| N-01 | Primary platform macOS 14+ (Apple Silicon **and** Intel — Intel is open, see `intel-display-backend`). Linux and Windows are future targets: only the display (DDC) backend differs per OS; the HID++ layer is shared. Backends declare which platforms they support; the app chooses at runtime. |
| N-02 | Rust (edition 2024), Cargo workspace. `cargo build` / `cargo test` / `cargo clippy` are the source of truth; no IDE-specific project files. |
| N-05 | No network, no telemetry, no secrets until `peer-state-and-remote-switch`. Tooling (OpenSpec CLI) is run with telemetry off (`OPENSPEC_TELEMETRY=0`). |
| N-06 | Distribution: **build from source** (`cargo build --release`). The project is licensed **Apache-2.0** (`LICENSE`); every source file carries an `SPDX-License-Identifier: Apache-2.0` header; dependencies must be compatible with Apache-2.0 (no bundled GPL code). There are no App Store, sandbox, signing or notarization requirements; private/platform-specific APIs and external tools are allowed. |
| N-07 | Core domain and wire protocol have no OS or UI dependency (`hostswitch-core` has no platform code and forbids `unsafe`), so Linux/Windows front-ends and peers can reuse them. |
| N-08 | Code and CLI output are vendor-neutral: no manufacturer name is hard-coded; names appear only where read from the hardware or stored in the user's configuration. |
| N-09 | Tests are separated from functional code per Rust conventions (see ADR-0011); test descriptions are Gherkin `.feature` files traced to OpenSpec scenarios. |

## 4. Constraints & known risks
- **R-1 External tools.** The current backends shell out to `m1ddc` and `hidapitester` (user-installed). The backend interfaces are the exit path to native implementations (`hidapi`, per-OS DDC), which also remove the GPL `hidapitester` dependency (change `native-backends`). Native DDC on macOS uses the private IOAVService API (Apple Silicon) / IOFramebuffer (Intel); acceptable because the app is not distributed through the App Store.
- **R-2 Bluetooth-only devices.** A device on Bluetooth is only reachable while it is connected to *this* computer. Switching *away* from the active host works; switching *to* this host from another host requires that other host to act (hence `peer-state-and-remote-switch`).
- **R-2a Several ways to reach one device (verified on hardware).** Specified in `device-aliases`.
- **R-3 DDC reliability.** Monitors differ in VCP codes and timing; retries and per-display input values are needed.
- **R-3a Available ports are not readable with m1ddc.** Specified in `discovery`; fixed by `native-backends`.
- **R-4 Intel Macs.** `m1ddc` is Apple-Silicon only; see `intel-display-backend`.
- **R-5 Permissions.** HID access needs Input Monitoring on macOS; the reaction is specified in `discovery`.
- **R-6 Offline devices.** Verified on hardware (docs/research/hidpp.md): an empty slot and an offline/sleeping device return the same receiver error, so presence can only be inferred; the program remembers devices it has seen (`offline-devices`).

## 5. Open questions
- Should switching *to* the current host be a no-op or re-assert? (Today it always re-asserts every step.)
- Hostname as key vs. a stable generated id (hosts already have a UUID; hostname is only a label for matching).
- Which hosts' profiles may run `init` for the other computers' display inputs? (`--host LABEL=INPUT`, user-provided.)
