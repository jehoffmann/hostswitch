# Device Aliases Specification

## Purpose
One physical keyboard or mouse can be reachable in different ways (Bluetooth in one Easy-Switch slot, a Unifying receiver in another). Aliases let one profile work in both directions.

## Requirements

### Requirement: R-2a Aliases are used when the primary id is offline
The system SHALL, for a keyboard or mouse step, try the device id of the profile first and then each of its aliases in order, and SHALL move on to the next id only when the previous one is offline. Any other error SHALL end the attempt.

#### Scenario: Primary offline, alias works
- **GIVEN** keyboard "kb" has alias "kb-usb" and "kb" is offline
- **WHEN** the user switches to a host using "kb"
- **THEN** the switch is sent to "kb-usb" and succeeds

#### Scenario: Primary works
- **GIVEN** keyboard "kb" with an alias
- **WHEN** the primary id answers
- **THEN** the alias is never used

#### Scenario: Real error is not masked
- **GIVEN** the primary id fails with an error other than offline
- **WHEN** the step runs
- **THEN** the step fails with that error and no alias is tried

#### Scenario: Every way is offline
- **GIVEN** the primary id and all aliases are offline
- **WHEN** the step runs
- **THEN** the step is reported as FAIL with "device offline"

### Requirement: R-2b Declaring and validating aliases
The system SHALL let the user declare an alias for a device and SHALL reject aliases that equal their own device, are blank, or collide with another device id or alias.

#### Scenario: Link a second way
- **GIVEN** a configuration
- **WHEN** the user links "046d:b359/255" with "046d:c52b/1"
- **THEN** the alias is saved and shown by the mapping output

#### Scenario: Conflicting alias
- **GIVEN** "a" already has alias "b"
- **WHEN** the user links "c" with "b"
- **THEN** the request is rejected and the configuration is unchanged

### Requirement: R-2c The current state is read through any way
The system SHALL read the current slot of a device through its id or any alias, the first that answers winning.

#### Scenario: Status while the device is on the receiver
- **GIVEN** the keyboard only answers through its alias
- **WHEN** the status is shown
- **THEN** the keyboard's current slot is shown
