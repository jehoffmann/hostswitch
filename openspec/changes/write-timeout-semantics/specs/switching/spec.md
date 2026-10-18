# Delta for switching


## ADDED Requirements

### Requirement: SEC-10 A write with an unknown outcome is not repeated
The system SHALL report a keyboard or mouse step whose write timed out as "unknown", SHALL NOT retry it, SHALL NOT send it to an alias, and SHALL tell the user to check the current state with `status`.

#### Scenario: Timeout during the write
- **GIVEN** the set-current-host write exceeds the step timeout
- **WHEN** the step runs
- **THEN** the step is reported as UNKNOWN with a hint to run `status`
- **AND** no second write is sent to any id

### Requirement: SEC-10a Timeouts fit the backend
The system SHALL use a step timeout at least as long as one complete attempt of the slowest backend.

#### Scenario: Bluetooth lookup and write
- **GIVEN** a Bluetooth device that answers slowly but within its read limits
- **WHEN** it is switched
- **THEN** the step is not cut off by the timeout
