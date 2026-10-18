# Delta for discovery


## ADDED Requirements

### Requirement: R-3b Real input ports are listed
The system SHALL, when the monitor reports its capabilities, list exactly the input ports the monitor supports and say so; otherwise it SHALL fall back to the standard list with the existing note.

#### Scenario: Monitor reports capabilities
- **GIVEN** a monitor whose capabilities list inputs 15, 17 and 27
- **WHEN** the user lists displays with ports
- **THEN** only DP1, HDMI1 and USB-C are listed and marked as reported by the monitor

### Requirement: N-03e Discovery stops at the matching reply
The system SHALL finish a request as soon as its reply arrives and SHALL NOT wait for further reads.

#### Scenario: Fast reply
- **GIVEN** a device answers within a few milliseconds
- **WHEN** it is pinged
- **THEN** the call returns without waiting for the read timeout
