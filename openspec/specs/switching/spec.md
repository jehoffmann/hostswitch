# Switching Specification

## Purpose
Move the desk to a host: set the display inputs, then the keyboard and the mouse Easy-Switch slot named in that host's profile, safely and with a per-step result.

## Requirements

### Requirement: F-01 Switch to a host profile
The system SHALL, when asked to switch to a host, apply every display input, the keyboard slot and the mouse slot named in that host's profile and report the result of each step. A host SHALL be selectable by label, hostname or id, case-insensitively.

#### Scenario: Full profile is applied
- **GIVEN** host "Work" has a profile with one display, a keyboard and a mouse
- **WHEN** the user switches to "Work"
- **THEN** the display input is set, the keyboard is switched to its slot and the mouse is switched to its slot
- **AND** each step is reported as OK

#### Scenario: Host is found by label, hostname or id
- **GIVEN** hosts "Work" with hostname "mbp.local" and "Home"
- **WHEN** the user names a host as "WORK", "mbp.local" or its id
- **THEN** the host "Work" is selected

#### Scenario: Unknown host or host without a profile
- **GIVEN** no host named "nobody", and a host "Bare" without a profile
- **WHEN** the user switches to "nobody" or to "Bare"
- **THEN** the system reports an error naming the problem
- **AND** no hardware is changed

### Requirement: F-01a Order of steps
The system SHALL apply steps in the order displays, then keyboard, then mouse.

#### Scenario: Cross-backend order
- **GIVEN** a profile with two displays, a keyboard and a mouse
- **WHEN** the plan is executed
- **THEN** both displays are set before the keyboard, and the keyboard before the mouse

### Requirement: F-03 Several displays
The system SHALL support any number of displays per profile, each with its own input.

#### Scenario: Five displays
- **GIVEN** a profile with five displays
- **WHEN** the user switches to that host
- **THEN** five display steps are planned, one per display

### Requirement: F-04 Optional parts
The system SHALL treat display, keyboard and mouse entries of a profile as optional and switch only what is present.

#### Scenario: Mouse missing
- **GIVEN** a profile with a display and a keyboard but no mouse
- **WHEN** the user switches to that host
- **THEN** only the display and the keyboard are switched

#### Scenario: Empty profile
- **GIVEN** a profile without displays, keyboard or mouse
- **WHEN** the user switches to that host
- **THEN** the system reports that there is nothing to switch and changes nothing

### Requirement: N-04a A failed display protects keyboard and mouse
The system SHALL NOT switch the keyboard or the mouse when a display step failed or timed out after its retries, and SHALL report them as skipped with a reason. A failed keyboard step SHALL NOT skip the mouse. Other display steps SHALL still run.

#### Scenario: Display fails
- **GIVEN** a profile with a display, a keyboard and a mouse, and the display cannot be set
- **WHEN** the user switches to that host
- **THEN** the display step is reported as FAIL
- **AND** the keyboard and mouse steps are reported as SKIP and no command is sent to them

#### Scenario: Keyboard fails
- **GIVEN** the keyboard cannot be switched
- **WHEN** the user switches to the host
- **THEN** the keyboard step is reported as FAIL and the mouse is still switched

#### Scenario: One of two displays fails
- **GIVEN** two displays of which the first cannot be set
- **WHEN** the user switches to the host
- **THEN** the second display is still set and the keyboard and mouse are skipped

### Requirement: N-04b Every step is time-bounded
The system SHALL bound every step by a timeout that holds even if the underlying operation ignores cancellation, and SHALL retry a failed step once before reporting failure.

#### Scenario: Step that never finishes
- **GIVEN** a display backend that blocks for longer than the step timeout
- **WHEN** the step is executed
- **THEN** the step is reported as failed with a timeout message within the timeout, not when the backend finally returns

#### Scenario: Transient failure
- **GIVEN** a display that fails once and then succeeds
- **WHEN** the step is executed
- **THEN** the step succeeds on the second attempt

### Requirement: N-04c Missing backend
The system SHALL report a step as failed without retrying when no backend is available for it.

#### Scenario: No display tool installed
- **GIVEN** no display backend is available
- **WHEN** the user switches to a host with a display
- **THEN** the display step is reported as failed with a "backend unavailable" reason after one attempt

### Requirement: F-01b Switching is the default, dry run changes nothing
The system SHALL perform the switch when asked to switch to a host. When asked for a dry run, it SHALL print the plan and SHALL NOT access any hardware or change any file.

#### Scenario: Dry run
- **GIVEN** a configured host "Work"
- **WHEN** the user switches to "Work" with the dry-run option
- **THEN** the plan is printed with one line per step and "nothing was changed"
- **AND** no tool is started

### Requirement: F-01c Result and exit status
The system SHALL print one result line per step (OK, FAIL or SKIP with the reason) and SHALL exit with status 0 when every step succeeded and 2 when any step failed or was skipped.

#### Scenario: Partial failure
- **GIVEN** a switch in which one step fails
- **WHEN** the switch finishes
- **THEN** the exit status is 2
