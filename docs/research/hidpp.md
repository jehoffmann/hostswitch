# HID++ 2.0 Easy-Switch research

Verification level: enumeration was run on the dev machine. Byte layouts below are from Solaar sources (confirmed: feature id constants) plus protocol knowledge (NOT yet verified on hardware or via a fetched authoritative page; mark UNVERIFIED items before implementing).

## Dev machine enumeration (hidapitester 0.7, hidapi 0.15.0)
- Logitech USB receiver `046D/C52B` (Unifying-class receiver) with 3 interfaces: if0 keyboard (0001/0006), if1 mouse etc., if2 vendor collections usagePage `0xFF00` usage `0x0001`, `0x0002`, `0x0004` = HID++ short (0x10), long (0x11), very long (0x12) report collections. No BLE Logitech device present; a "Magic Mouse" (Apple) and MSI controller also listed.
- Fixtures: `hidapitester-list-all.txt`, `hidapitester-list-detail-receiver.txt`. Note device names contain non-UTF8 bytes (Magic Mouse umlaut): decode lossy.
- Receiver exposes no serial (`serial_number` empty); identify by path, usagePage 0xFF00 + usage.

## Protocol
- Report ids: 0x10 short (7 bytes total), 0x11 long (20), 0x12 very long (64).
- Layout: `[reportId, deviceIndex, featureIndex, (function<<4)|swId, params...]`.
- deviceIndex: 0xFF for a directly connected BLE/USB-wired device (UNVERIFIED for macOS BLE); 0x01..0x06 for devices paired to a Unifying/Bolt receiver (slot number of the paired device). Receiver itself is 0xFF for HID++ 1.0 register access.
- Feature lookup: root feature index is always 0x00. Request `getFeature` = function 0: params = feature id (big-endian 2 bytes), reply param[0] = feature index (0 means unsupported). e.g. `10 01 00 0X 18 14 00` (X = swId, use nonzero, e.g. 0x1... 0x0A) to find CHANGE_HOST (0x1814).
- CHANGE_HOST (0x1814) (Solaar `SupportedFeature.CHANGE_HOST = 0x1814`, confirmed; sibling `HOSTS_INFO = 0x1815` is for names):
  - fn 0 `getHostInfo`: reply `nbHosts, currentHost (0-based), flags` (UNVERIFIED layout). Read-only and safe: this is the way to read the current slot.
  - fn 1 `setCurrentHost(hostIndex)`: param[0] = 0-based host (slot 1 -> 0). The device switches and drops the connection; typically NO reply is returned. Do not send in tests.
  - Slot numbers in our config are 1..3, wire value is slot-1.
- Safe read example (does not switch): `hidapitester --vidpid 046D/C52B --usagePage 0xFF00 --usage 0x0002 --open --length 20 --send-output 0x11,0x01,0x00,0x0A,0x18,0x14,0x00,... --timeout 500 --read-input` (hidapitester sends output reports; first byte = report id; pad to 20 bytes). Untested here.
- Because the receiver multiplexes devices, listing devices requires pinging deviceIndex 1..6 (root fn 1 `getProtocolVersion`, reply echoes ping) and reading name (feature 0x0005 DEVICE_NAME_TYPE) for kind (keyboard/mouse).

## Verified on hardware (2026-10-18, Unifying receiver 046D/C52B, read-only commands run by the user)
Fixtures: `Tests/BackendsTests/Fixtures/hidpp-*.txt`.
- Requests as **short** reports (`0x10`, 7 bytes) on usage `0x0001` work; the receiver may answer with a **long** report (`0x11`), so replies must be accepted as 0x10/0x11/0x12.
- `hidapitester` prints `read N bytes:` + one hex line. A timeout prints `read 0 bytes:` + zeros: that is *no reply*, not a report.
- Ping (`10 ii 00 1A 00 00 5A`): a real device echoes `11 ii 00 1A 04 05 5A` (HID++ 4.5). Slots 1 and 2 were live, slots 3-6 empty.
- Empty slot: HID++ 1.0 error `10 ii 8F 00 1A 09 00` (`8F` = error, code `09` = resource error). Must not be treated as a device.
- The first request to slot 1 once got no reply (device waking?); a retry works, so silence is retried once.
- `getFeature(0x1814)` on slot 1: reply `11 01 00 0A 09 00 01` -> CHANGE_HOST is at feature index **9**, version 1.
- `getHostInfo` to slot 1 (`11 01 09 0A ...`) got the receiver error `10 01 8F 09 0A 09 00` (resource error) although slot 1 had answered a ping minutes earlier. Error `09` is therefore ambiguous: *empty slot* **or** *paired but asleep/offline*. Devices appear to sleep; the first request wakes them (matches the silent first ping). Plan: retry with a short delay after a wake request. Fixture: `hidpp-gethostinfo-resource-error.txt`.
- Still **unverified**: `getHostInfo` reply layout (`11 01 09 0A ...`), device kind/name, BLE.
- `--nonexclusive` and flag order make no difference (checked with and without). Empty `hostswitchctl list devices` was correct: at test time all slots answered error `09` (devices asleep or connected to another host). Detection must treat that as "offline", not "absent".

## Verified on hardware: Bluetooth LE (2026-10-18, ERGO K860 `046D/B359`, MX Vertical `046D/B020`)
Fixtures: `hidpp-ble-*.txt`, `hidapitester-list-detail-ble.txt`.
- macOS exposes each Bluetooth device as several HID collections with `bus_type: Bluetooth`; the HID++ channel is usagePage `0xFF43`, usage `0x0202`. Use **long** reports (`0x11`, 20 bytes) and device index **`0xFF`**.
- Ping `11 FF 00 1A 00 00 5A ...` -> `11 FF 00 1A 04 05 5A ...` (HID++ 4.5). `getFeature(0x1814)` on the K860 -> `11 FF 00 0A 08 00 01 ...`: CHANGE_HOST is at feature index **8**, version 1.
- The same input stream carries ordinary HID reports (mouse movement `02 00 00 xx xx ...`). A moving MX Vertical delivered the reply as the **3rd** report, so replies must be matched (device index + feature index + function/software id, or an error report) instead of taking the first report. `hidapitester` performs every `--read-input` even after the reply, so the backend issues 5 reads of 400 ms; a native `hidapi` backend can stop at the match.
- Opening the Bluetooth collection requires **Input Monitoring** for the calling process; without it `hidapitester` prints `hid_open_path ... (0xE00002E2) not permitted`. The backend reports this as `PermissionDenied` with instructions. USB receiver vendor collections opened without it.
- Names and kinds for Bluetooth devices come from the enumeration (description "Logitech - ERGO K860"; Generic Desktop keyboard usage `0x01/0x06` = keyboard, else pointer `0x01/0x02` = mouse). Receiver devices still need the DEVICE_NAME_TYPE (0x0005) query (TODO).
- `hidapitester` prints its errors (`Error: no HID devices found for given vid/pid`, `... not permitted`) on **stdout**, not stderr, and exits 1. After a Bluetooth device switched to another host it disappears from the enumeration and opening it fails with the first message; the backend maps that to "device offline" so the receiver id can take over.
- Verified via the Unifying receiver (device on slot 3): `getHostInfo` reply `11 02 0C 0A 03 02 00` = 3 hosts, current host index 2 (0-based) = slot 3.
- Still **unverified**: `setCurrentHost` on Bluetooth and via the receiver (not run: it would switch hardware).

## Bolt vs Unifying vs BLE
- Bolt (046D/C548) and Unifying (C52B): same HID++ reports via the receiver vendor collection, deviceIndex = paired slot.
- BLE direct: macOS exposes the HID++ collection of BLE HID devices, but macOS may restrict raw access to Bluetooth keyboards/mice (a third-party note says macOS blocks raw HID access to Bluetooth input devices; Logi Options+ uses its own agent). UNVERIFIED; treat BLE as high risk and test on real device read-only.
- R-2 applies: a BLE device can only be commanded while connected to this Mac.

## Permissions / sandbox
- Opening keyboard/mouse HID devices needs Input Monitoring (TCC) for the host app/terminal. Vendor-defined usagePage 0xFF00 collections on the receiver may open without it; keyboard (0001/0006) interfaces will prompt. `--nonexclusive` option exists in hidapitester.

## Sources
- https://github.com/pwr-Solaar/Solaar/blob/master/lib/logitech_receiver/hidpp20_constants.py (CHANGE_HOST=0x1814, HOSTS_INFO=0x1815, confirmed)
- https://github.com/pwr-Solaar/Solaar/blob/master/lib/logitech_receiver/hidpp20.py (request pattern, HOSTS_INFO functions)
- Search hint (not opened): OpenLogi `change_host.rs`, https://github.com/Phecda/OpenLogi ; logitech-ipc-protocol notes on macOS BLE.

## Go / no-go
- hidapitester: GO for the MVP/dev prototype (read-only enumeration works; one-shot send/read is enough, ~100 ms per call). Weaknesses: GPL-3.0, each call re-opens the device, awkward response matching/timeouts, no async notifications.
- Native HID backend (`hidapi` crate): GO for production (GPL-free, reliable request/response matching). Recommended path: ship hidapitester backend first behind `PeripheralBackend`, then replace with native IOHIDManager backend using the same byte layouts.
- Before implementation, verify on hardware read-only: feature index of 0x1814 and getHostInfo reply layout on the user's keyboard/mouse (model unknown, open question).
