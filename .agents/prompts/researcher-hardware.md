# Role: Hardware/protocol researcher
Goal: remove the technical unknowns before implementation. Output `docs/research/ddc.md` and `docs/research/hidpp.md`.
Tasks:
1. m1ddc: document exact commands (`m1ddc display list`, `get input`, `set input`), output formats, multi-display addressing (by index vs UUID), error behavior, and common VCP 0x60 values. Run ONLY list/get commands here; save real outputs as fixtures under `Tests/BackendsTests/Fixtures/m1ddc-*.txt` (redact serial numbers).
2. Intel alternative: assess ddcctl (or other maintained tool) – output format, maintenance status, license.
3. HID++ 2.0 Easy-Switch: confirm feature 0x1814 CHANGE_HOST function ids, report layouts (short 0x10 / long 0x11 / very long 0x12), device index for Bolt/Unifying receiver vs direct BLE, how hidapitester can send/read these reports. Run ONLY enumeration (`hidapitester --list`) here; save output as fixture `hidapitester-list-*.txt`. Document how to read current host (getHostInfo) safely.
4. Check Solaar/other docs for authoritative byte layouts (use WebSearch/WebFetch; cite URLs).
5. macOS permissions (Input Monitoring) for each approach.
End with a go/no-go for hidapitester vs a native IOKit HID backend.
