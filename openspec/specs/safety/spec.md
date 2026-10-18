# Safety Specification

## Purpose
hostswitch only switches channels. It must never send a command that could damage or disable a monitor or peripheral, and it must not misuse the system it runs on.

## Requirements

### Requirement: S-01 Only channel switching reaches the hardware
The system SHALL send to a display only the commands that read or set the input source, and to a keyboard or mouse only the requests that ping, look up a feature, read the current host and set the current host. No other display or device command SHALL be reachable.

#### Scenario: Switching a keyboard
- **WHEN** a keyboard is switched
- **THEN** the only write sent to it is the set-current-host request, with the slot of the profile

### Requirement: S-02 No write without a successful lookup
The system SHALL send a set-current-host request only after the device's feature lookup succeeded, and SHALL send no write when the lookup fails, is unsupported, errors or cannot be parsed.

#### Scenario: Lookup fails
- **GIVEN** the feature lookup returns an error, silence, garbage or "unsupported"
- **WHEN** a keyboard switch is attempted
- **THEN** no set-current-host request is sent

### Requirement: S-03 Reading never writes
The system SHALL NOT send any write request when only reading the current state.

#### Scenario: Status
- **WHEN** the current host of a device is read
- **THEN** only read requests are sent

### Requirement: S-04 External tools are started without a shell
The system SHALL start external tools with their arguments as separate elements, never through a shell, with a timeout, and SHALL kill a tool that exceeds it.

#### Scenario: Hostile characters in an argument
- **GIVEN** an argument containing shell metacharacters
- **WHEN** a tool is started
- **THEN** the tool receives the argument verbatim

#### Scenario: Tool hangs
- **GIVEN** a tool that does not exit
- **WHEN** its timeout elapses
- **THEN** it is killed and a timeout error is reported

### Requirement: S-05 Dry run is honest
The system SHALL NOT start any tool when a dry run is requested.

#### Scenario: Dry run
- **WHEN** a switch is requested with the dry-run option
- **THEN** no display or device backend is used
