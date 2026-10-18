# DDC research (m1ddc / Intel alternatives)

Verified on dev machine: arm64, m1ddc 1.2.0 (Homebrew, MIT), display MSI MPG341CQR. Only list/get commands were run.

## m1ddc commands
| Purpose | Command | Output |
|---|---|---|
| list | `m1ddc display list` | `[1] MSI MPG341CQR (1C95A13D-...)`: `[index] name (system UUID)`, one line per display |
| detailed list | `m1ddc display list detailed` | key/value block (Product name, Vendor, Model, Serial, Display ID, System UUID, EDID UUID, IO Location, Adapter) |
| get input | `m1ddc display 1 get input` | bare integer, e.g. `15` (DP1), exit 0 |
| set input | `m1ddc display 1 set input 17` | no output (NOT run here) |
| alt addressing (LG) | `set input-alt n` | DP1=208, DP2=209, HDMI1=144, HDMI2=145, USB-C=210 |

VCP 0x60 values (m1ddc help): DP1=15, DP2=16, HDMI1=17, HDMI2=18, USB-C=27. Per-monitor deviations exist (R-3), so values must be user-overridable. MSI MPG341CQR currently reports 15.

## Addressing
- `display <n>` (index, 1-based, order can change when displays are plugged/unplugged) or `display <method>=<id>`; default method is `uuid` (system UUID, stable per connection). Methods: id, uuid, edid, seid, basic, ext, full. `display uuid=<UUID> get luminance` verified working.
- Recommendation: store the EDID-based identity (`edid` or `basic` vendor:model:serial) in config and resolve to the current UUID/index via `display list detailed` at switch time.
- Serials/EDID UUID are redacted in fixtures.

## Errors / reliability
- Unknown display: stderr `The specified display does not exist. Use 'display list' ...`, exit 1 (fixture `m1ddc-error-no-display.txt`).
- OBSERVED FLAKINESS: the very first `get input` call returned empty output with exit 1 (monitor probably in a wake/idle state); five subsequent calls returned `15`, exit 0. Treat empty stdout or non-zero exit as transient: retry with backoff (e.g. 3x, 300 ms) and treat empty output as failure, never as "no input". Supports N-04.
- m1ddc does not support the built-in HDMI port of base M1/M2 Macs; USB-C/DP-alt only (README text).

## Intel alternative
- m1ddc is Apple Silicon only. ddcctl (kfix/ddcctl, MIT license): `ddcctl -d 1 -i 15` sets input; `-i ?` style read is limited and output is free-form text. Maintenance is sparse and Intel-only in practice. Not tested here (arm64 machine). Verify license/maintenance online before committing to it; native IOFramebuffer/I2C backend is the long-term option. Confidence: moderate, from memory, not verified in this session.

## Fixtures (Tests/BackendsTests/Fixtures)
m1ddc-display-list.txt, m1ddc-display-list-detailed.txt (redacted), m1ddc-get-input.txt, m1ddc-error-no-display.txt.

## Native implementation
m1ddc uses the private IOAVService API. A native backend can use the same calls via FFI (kept out of `hostswitch-core`). See R-1.
