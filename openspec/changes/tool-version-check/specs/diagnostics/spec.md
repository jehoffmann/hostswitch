# Delta for diagnostics


# Diagnostics Specification

## Purpose
Tell the user which helper tools are used and whether they are usable.

## ADDED Requirements

### Requirement: N-03d Tool paths and versions are reported
The system SHALL report, for each helper tool, the resolved path and the version, and SHALL flag a tool that is missing or whose version is outside the supported range.

#### Scenario: Supported version
- **GIVEN** a tool in the supported range
- **WHEN** the diagnostics are shown
- **THEN** path and version are listed without a warning

#### Scenario: Missing tool
- **GIVEN** a tool that is not installed
- **WHEN** the diagnostics are shown
- **THEN** it is listed as not found with a hint how to install it
