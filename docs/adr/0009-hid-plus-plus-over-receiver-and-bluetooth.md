# ADR-0009: HID++ over USB receivers and Bluetooth LE

- Status: Accepted
- Date: 2026-10-18

## Context
Switching a keyboard or mouse uses the HID++ 2.0 feature CHANGE_HOST (0x1814). Receiver devices and Bluetooth devices expose different HID collections, and a Bluetooth device's input stream also carries ordinary reports (mouse movement). Behaviour was verified on hardware (docs/research/hidpp.md).

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- spec:discovery/F-05a — receiver and Bluetooth devices
- spec:discovery/F-05b — names and kinds from the hardware
- spec:discovery/N-03a — replies are matched to requests
- spec:discovery/R-5 — permission problems are explained
- spec:safety/S-02 — no write without a successful lookup
- spec:safety/S-03 — reading never writes
- change:verify-device-identity/SEC-12 — exact reply matching

## Decision
The channel follows the device index in the id. **Receiver** (index 1 to 6): short reports (7 bytes) on usage page 0xFF00, usage 0x0001; replies may be short or long. **Bluetooth** (index 255): long reports (20 bytes) on usage page 0xFF43, usage 0x0202, several reads per request (5 reads of 400 ms) because unrelated input reports share the stream. A reply is accepted only if it is a HID++ report for the same device index whose feature and function bytes echo the request, or an error report for that device. A write (`setCurrentHost`, wire value slot-1) is sent only after a successful `getFeature` lookup of 0x1814; reading the current host sends no write. Names and the keyboard or mouse kind of Bluetooth devices come from the enumeration (manufacturer prefix cut at the first " - "). An Input Monitoring denial is reported with instructions (`PermissionDenied`), and receiver devices are still listed.

## Consequences
- Works on the verified hardware (ERGO K860, MX Vertical, Unifying receiver); Bolt receivers and other models are unverified.
- Reply matching can be made stricter (change `verify-device-identity`).
- A native HID library can stop at the matching reply instead of waiting (`native-backends`).

## Alternatives considered
- Treat the first report as the reply: failed on a moving mouse, whose reply came third.
