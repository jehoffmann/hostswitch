## Why
A Bluetooth keyboard or mouse can only be switched while it is connected to this computer (R-2): to bring it back, the other computer has to act. F-08, F-10 and F-11 describe sharing state and asking a peer to switch, but there is no network code and no security design for it.

## What Changes
- Each instance advertises its state on the local network and shows the others read-only.
- An instance can ask the instance that currently owns the devices to switch them.
- Peers are paired once out of band; every request is authenticated and protected against replay.

## Capabilities
### New Capabilities
- `peers`

### Modified Capabilities
- None

## Impact
New networking module behind a transport trait (core stays free of OS code); new dependencies for mDNS and a MAC; a key store. Makes hardened configuration storage (`harden-config-storage`) a hard prerequisite.

## Out of Scope
Internet access, accounts, any cloud service.
