# Config Specification

## Purpose
Hosts, profiles, remembered devices and aliases are stored in one JSON file that is validated, never silently corrupted and readable by humans.

## Requirements

### Requirement: F-02a Location of the configuration
The system SHALL use `$XDG_CONFIG_HOME/hostswitch/config.json` (relative values ignored), else `~/.config/hostswitch/config.json` on macOS and Linux, and `%APPDATA%\Hostswitch\config.json` on Windows, unless a path is given on the command line.

#### Scenario: Default and override
- **GIVEN** XDG_CONFIG_HOME is unset and HOME is "/home/u"
- **WHEN** the default path is resolved
- **THEN** it is "/home/u/.config/hostswitch/config.json"

#### Scenario: Relative XDG value
- **GIVEN** XDG_CONFIG_HOME is "relative"
- **WHEN** the default path is resolved
- **THEN** the value is ignored and the HOME based path is used

### Requirement: F-02b Missing file means empty configuration
The system SHALL treat a missing configuration file as an empty configuration.

#### Scenario: First run
- **GIVEN** no configuration file
- **WHEN** the configuration is shown
- **THEN** it reports that no hosts are configured

### Requirement: F-02c Versions and migration
The system SHALL load schema version 2, SHALL migrate version 1 files (no remembered devices) to version 2 on load, and SHALL reject any other version and a missing version with a readable error.

#### Scenario: Version 1 file
- **GIVEN** a version 1 file with hosts and profiles
- **WHEN** it is loaded
- **THEN** it loads as version 2 with an empty list of remembered devices

#### Scenario: Version 3 file
- **GIVEN** a file with version 3
- **WHEN** it is loaded
- **THEN** it is rejected with an "unsupported configuration version" error

### Requirement: F-02d Validation
The system SHALL reject configurations with duplicate host ids, labels (case-insensitive) or hostnames (blank ignored, case and space insensitive), blank labels, duplicate remembered devices, profiles for unknown hosts, several profiles for one host, a display listed twice, keyboard and mouse using the same device, slots outside 1 to 3 and display inputs outside 0 to 65535.

#### Scenario: Duplicate label
- **GIVEN** two hosts labelled "A" and "a"
- **WHEN** the configuration is validated
- **THEN** it is rejected

#### Scenario: Slot out of range
- **GIVEN** a profile with keyboard slot 4
- **WHEN** the configuration is loaded
- **THEN** it is rejected with a readable error

#### Scenario: Malformed input
- **GIVEN** an empty, truncated or wrongly typed JSON file
- **WHEN** the configuration is loaded
- **THEN** a readable "invalid configuration" error is reported

### Requirement: F-02e Saving
The system SHALL validate a configuration before saving it and SHALL write it atomically so that a rejected save leaves the previous file untouched.

#### Scenario: Rejected save
- **GIVEN** a valid configuration on disk
- **WHEN** an invalid configuration is saved
- **THEN** the save fails and the file on disk is unchanged

### Requirement: F-02f Compatibility with files written by earlier versions
The system SHALL read configuration configuration files written by earlier versions (same keys and types).

#### Scenario: File written by an earlier version
- **GIVEN** a file as written by an earlier version
- **WHEN** it is loaded
- **THEN** hosts, profiles, devices and slots are read correctly
