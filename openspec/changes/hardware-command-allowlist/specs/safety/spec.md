# Delta for safety


## ADDED Requirements

### Requirement: S-06 Closed set of hardware commands
The system SHALL be able to send only these hardware commands: read the input source of a display, set the input source of a display, ping a device, look up a feature, read the current host of a device and set the current host of a device. There SHALL be no code path that sends other text or bytes to a display tool or a device.

#### Scenario: Request building
- **WHEN** every request builder is exercised with hostile input
- **THEN** each produced request is one of the allowed commands or an error

### Requirement: S-07 Values come from the profile and are range-checked
The system SHALL send a display input value only if it comes from the user's profile and lies within the valid input range, and a host slot only if it lies within 1 to 3.

#### Scenario: Value from tool output
- **GIVEN** a tool prints a number as a display's current input
- **WHEN** a switch is executed
- **THEN** that number is never used as a value to set

### Requirement: S-08 Write frequency is bounded
The system SHALL send at most one write per device and display per switch plus the single permitted retry, and SHALL refuse repeated switches to the same host within a short guard interval unless the user forces it.

#### Scenario: Repeated invocation
- **GIVEN** a switch to "Work" ran a moment ago
- **WHEN** the user runs it again immediately
- **THEN** the second run is refused with a message unless forced
