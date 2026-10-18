# ADR-0013: Peer state over mDNS and authenticated remote switching

- Status: Proposed
- Date: 2026-10-18

## Context
A Bluetooth device can only be switched while it is connected to the computer that runs the tool (R-2). To bring it back, the other computer has to act, so instances must find each other and be able to ask each other.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- change:peer-state-and-remote-switch/F-08 — state advertised read-only
- change:peer-state-and-remote-switch/F-11 — conflict-free state
- change:peer-state-and-remote-switch/F-10 — authenticated remote switch
- change:peer-state-and-remote-switch/F-10a — pairing and key storage
- change:peer-state-and-remote-switch/F-10b — network abuse is bounded
- constraint:N-05 — no network or secrets until this change

## Decision
mDNS service `_hostswitch._tcp` with a small TXT record (protocol version, host id, label, revision); state messages are platform-neutral JSON (host id, label, revision, displays, devices). Each instance is the only writer of its own record and a higher revision wins. A remote switch request is authenticated with an HMAC made from a per-pair 256-bit key created at an explicitly confirmed pairing, carries a nonce and a timestamp window, may only switch the desk to the requesting peer, and never names a display id, device id or slot: the receiver runs its own local plan. Keys are stored in the keychain or a private file. The code lives in `hostswitch-core` behind a transport trait; cryptography uses reviewed crates. The security requirements (authentication, replay protection, authorization, key storage, abuse limits) are part of the change proposal.

## Consequences
- Requires hardened configuration storage (change `harden-config-storage`) and trusted tool binaries (change `trusted-tool-binaries`) before any key is stored.
- New dependencies for mDNS and a MAC; every message is untrusted input.

## Alternatives considered
- A cloud relay: no LAN-only guarantee, accounts and a server to run.
- Unauthenticated switching on the LAN: anyone on the network could move the desk.
