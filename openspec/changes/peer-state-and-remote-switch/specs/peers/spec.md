# Delta for peers


# Peers Specification

## Purpose
Instances on the same local network share their state and can ask each other to switch devices that only the owning computer can reach.

## ADDED Requirements

### Requirement: F-08 State is advertised read-only
The system SHALL advertise this instance's host id, label and state revision on the local network and SHALL show the state of other instances read-only. A received state SHALL NOT be trusted for any action.

#### Scenario: Peer appears
- **GIVEN** another instance on the network
- **WHEN** the user opens the peer view
- **THEN** its label and its devices' current slots are shown as informational

### Requirement: F-11 Conflict-free state
The system SHALL accept a peer's state only when its revision is higher than the stored one for that peer, and each instance SHALL be the only writer of its own record.

#### Scenario: Old state
- **GIVEN** state revision 7 is stored for a peer
- **WHEN** a message with revision 5 arrives
- **THEN** it is ignored

### Requirement: F-10 Authenticated remote switch
The system SHALL execute a remote switch request only if it carries a valid authentication code made with the key shared at pairing, a timestamp within a short window and a nonce not seen before, and only to switch the requested devices to the requesting peer's own profile.

#### Scenario: Valid request
- **GIVEN** two paired instances
- **WHEN** the peer asks to switch the keyboard to its slot
- **THEN** the keyboard is switched to that slot

#### Scenario: Replayed request
- **GIVEN** a request was executed
- **WHEN** the identical request arrives again
- **THEN** it is rejected

#### Scenario: Unpaired or tampered request
- **GIVEN** a request without a valid code
- **WHEN** it arrives
- **THEN** it is rejected without any hardware access

### Requirement: F-10a Pairing and key storage
The system SHALL create a 256-bit random key per pair of instances during an explicit pairing step confirmed on both computers, and SHALL store keys readable only by the user (or in the system keychain).

#### Scenario: Key file
- **WHEN** a pairing completes
- **THEN** the key is stored with owner-only permissions and never printed or logged

### Requirement: F-10b Network abuse is bounded
The system SHALL limit message size, request rate, number of peers and the work done per unauthenticated message, and SHALL NOT send tool output, paths or error details to peers.

#### Scenario: Flood
- **GIVEN** a peer sends requests faster than the limit
- **WHEN** the limit is exceeded
- **THEN** further requests are dropped without hardware access
