# Delta for tools


## ADDED Requirements

### Requirement: SEC-6 Only absolute directories are searched
The system SHALL ignore empty and relative entries of PATH when resolving a tool and SHALL NOT run a tool from the current directory.

#### Scenario: Empty PATH entry
- **GIVEN** a PATH with an empty entry and a tool of that name in the current directory
- **WHEN** the tool is resolved
- **THEN** the tool in the current directory is not used

### Requirement: SEC-8 Output is bounded
The system SHALL stop reading a tool's standard output and error after a fixed limit, kill the tool and report an error.

#### Scenario: Endless output
- **GIVEN** a tool that writes without end
- **WHEN** it is run
- **THEN** memory use stays bounded and an error reports the output limit

### Requirement: SEC-9 A timeout ends the whole process group
The system SHALL terminate the tool and every process it started when a timeout elapses.

#### Scenario: Wrapper script
- **GIVEN** a tool that starts a long-running child
- **WHEN** the timeout elapses
- **THEN** neither the tool nor its child is running afterwards

### Requirement: SEC-7 No program is started outside the runner
The system SHALL start every external program, including the one used to find this computer's name, through the same runner with a timeout and the same path rules.

#### Scenario: Hostname lookup hangs
- **GIVEN** the hostname program does not return
- **WHEN** a configuration is initialised
- **THEN** the lookup times out and a default name is used
