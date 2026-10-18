# CLI Specification

## Purpose
`hostswitchctl` exposes every operation for scripting and humans with a consistent, vendor-neutral output.

## Requirements

### Requirement: F-07 Commands
The system SHALL provide the commands `list` (displays, devices, or both, with `--ports`), `show`, `status`, `init`, `link` and `switch`, and a global `--config <path>` option.

#### Scenario: Help
- **WHEN** the user runs the tool with `--help`
- **THEN** all commands are listed and the exit status is 0

### Requirement: F-07a Exit statuses
The system SHALL exit with 0 on success, 1 on an operational error, 2 when a switch only partly succeeded and 64 on invalid usage.

#### Scenario: Unknown host
- **WHEN** the user switches to an unknown host
- **THEN** the exit status is 1 and the error names the host

#### Scenario: Missing argument or unknown command
- **WHEN** the user omits the host of `switch` or enters an unknown command
- **THEN** the exit status is 64

### Requirement: F-07b Show prints the configuration only
The system SHALL print each host with its profile (Display, Keyboard, Mouse, with names, slots and aliases) from the configuration file without accessing hardware.

#### Scenario: Two hosts
- **GIVEN** hosts "Mac" and "PC" with profiles
- **WHEN** the user runs `show`
- **THEN** each host is printed with Display, Keyboard and Mouse lines whose values are aligned
- **AND** hosts without a profile are marked "(no profile)" and hosts with an empty profile "(empty profile)"

#### Scenario: No hosts
- **GIVEN** an empty configuration
- **WHEN** the user runs `show`
- **THEN** a hint to run `init` is printed

### Requirement: F-07c Status compares the configuration with the hardware
The system SHALL print each configured host with its profile and, per value, OK when the hardware already matches, "now: <value>" when it differs and "now: unknown" when it could not be read, and SHALL mark the active host with `*`.

#### Scenario: Mac active
- **GIVEN** the display and devices are in the state of host "Mac"
- **WHEN** the user runs `status`
- **THEN** "Mac" is marked with `*` and its values are OK
- **AND** the values of "PC" show the current state, for example "now: Slot 1"

### Requirement: F-07d Init builds a starting configuration
The system SHALL build a configuration from the detected displays (current inputs) and online keyboard and mouse (current slots), give this computer its short hostname as label, give each `--host LABEL[=INPUT]` the next free Easy-Switch slot (at most two other hosts) and print it. It SHALL write the file only with `--write`, and SHALL replace existing hosts only with `--force`, always keeping remembered devices.

#### Scenario: Print only
- **WHEN** the user runs `init` without `--write`
- **THEN** the configuration is printed and the file is not changed

#### Scenario: Existing hosts are protected
- **GIVEN** a configuration with hosts
- **WHEN** the user runs `init --write` without `--force`
- **THEN** the command fails and the file is unchanged

#### Scenario: Too many hosts
- **WHEN** the user passes three `--host` options
- **THEN** the command fails because Easy-Switch has three slots

#### Scenario: Unknown input name
- **WHEN** the user passes `--host PC=dp9`
- **THEN** the exit status is 64 and the valid names are listed

### Requirement: F-07e Link declares an alias
The system SHALL provide `link <device> <also>` which saves an alias and fails without changes on a conflict.

#### Scenario: Link
- **WHEN** the user runs `link a b`
- **THEN** "a can now also be reached as b" is printed and the alias is saved

### Requirement: F-07f Output style
The system SHALL print capitalised labels (Display, Keyboard, Mouse), names with their id in parentheses, aligned values, "Slot N" and OK, FAIL and SKIP for results, and SHALL NOT contain a hard-coded manufacturer name in its own output.

#### Scenario: Plan and report lines
- **GIVEN** a switch with a display, a keyboard and a mouse
- **WHEN** the plan and the report are printed
- **THEN** lines look like "1. Display:  <id> -> DP1 (15)" and "OK    Keyboard: ERGO K860 (<id>) -> Slot 2"
