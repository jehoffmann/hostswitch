# Tools Specification

## Purpose
External helper programs (m1ddc, hidapitester) are found, run with a timeout and their output is parsed in isolation.

## Requirements

### Requirement: N-03 Resolve and run external tools
The system SHALL resolve a tool from an explicit override, then the usual Homebrew directories, then PATH, SHALL run it with a timeout, SHALL capture standard output and error in full without deadlocking on large output, and SHALL decode them leniently.

#### Scenario: Tool is missing
- **GIVEN** a tool that is not installed
- **WHEN** it is needed
- **THEN** a "backend unavailable" error names the tool

#### Scenario: Large output
- **GIVEN** a tool that writes 300 KB to both outputs
- **WHEN** it is run
- **THEN** all output is captured and the call returns

#### Scenario: Non-UTF-8 output
- **GIVEN** a tool that prints a byte that is not valid UTF-8
- **WHEN** it is run
- **THEN** the output is decoded leniently and the call succeeds

### Requirement: N-03b Display backend behaviour
The system SHALL read the current display input with up to three attempts, treating empty output as a failure and never as "no input", and SHALL use the display's stable system UUID as its id.

#### Scenario: Empty first answer
- **GIVEN** the tool answers empty once and then "15"
- **WHEN** the input is read
- **THEN** the input 15 is returned after two attempts

### Requirement: N-03c Parsers tolerate garbage
The system SHALL parse tool output in isolated functions that return nothing for garbage, truncated, oversized or oddly encoded input and never panic.

#### Scenario: Garbage list
- **GIVEN** garbage instead of a display list
- **WHEN** it is parsed
- **THEN** no displays are returned
