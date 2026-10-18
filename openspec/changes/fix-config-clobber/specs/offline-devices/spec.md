# Delta for offline-devices


## ADDED Requirements

### Requirement: SEC-1 Remembering devices never destroys the configuration
The system SHALL remember discovered devices only when the configuration was loaded successfully (or does not exist), SHALL leave an unloadable configuration untouched, and SHALL report a failed save instead of ignoring it.

#### Scenario: Configuration of a newer version
- **GIVEN** a configuration file with an unsupported version
- **WHEN** the user lists devices
- **THEN** the devices are listed, a warning explains why they are not remembered
- **AND** the file is byte-for-byte unchanged

#### Scenario: Save fails
- **GIVEN** the configuration directory is read-only
- **WHEN** the user lists devices
- **THEN** the devices are listed and a warning reports that remembering failed
