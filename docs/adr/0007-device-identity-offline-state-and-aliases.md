# ADR-0007: Device ids, remembered devices and aliases

- Status: Accepted
- Date: 2026-10-18

## Context
A keyboard or mouse is reached through a USB receiver slot or directly over Bluetooth, and the same device may be paired both ways in different Easy-Switch slots. It may be asleep or connected to another computer, and an empty receiver slot looks like an asleep device.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- spec:offline-devices/F-13 — remember devices, show silent ones as offline
- spec:offline-devices/F-13a — profiles may use offline devices
- spec:offline-devices/F-13b — offline switching retried once
- spec:offline-devices/F-13c — a device that left the computer is offline
- spec:device-aliases/R-2a — aliases used when the primary id is offline
- spec:device-aliases/R-2b — declaring and validating aliases
- spec:device-aliases/R-2c — state read through any way
- spec:discovery/F-05a — online only on a genuine ping
- change:validate-identifiers/SEC-4 — canonical device ids
- change:verify-device-identity/SEC-4a — check the device behind an id

## Decision
A device id is `vid:pid/index` in lower-case hex: index 1 to 6 is a receiver slot, 255 is a directly connected Bluetooth device. A device is `online` only if it answered a genuine ping; receiver error 09 and silence are not reported as devices. Devices seen online are remembered (`knownDevices`, with name, kind, transport) and listed as `offline` when silent; profiles may reference them. `deviceAliases` maps the id used in profiles to other ids of the same physical device; the executor tries the primary id first and moves to the next id only on `DeviceOffline`, so other errors are never masked; `status` reads the slot through any id. A Bluetooth device that disappeared from the system counts as offline.

## Consequences
- One profile works in both directions (Bluetooth slot and receiver slot of the same machine).
- A receiver index names a slot, not a physical device; id handling is being tightened (changes `validate-identifiers` and `verify-device-identity`).

## Alternatives considered
- Identify devices by serial number or unit id: not available over every transport without extra protocol work (name query is still open).
- One entry per transport in profiles: doubles the configuration and cannot express the same device in two slots.
