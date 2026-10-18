# Delta for config


## ADDED Requirements

### Requirement: SEC-14 Unknown fields survive a save
The system SHALL keep top-level and nested fields it does not know when it saves the configuration.

#### Scenario: Field from a newer tool
- **GIVEN** a configuration with an extra field "futureOption"
- **WHEN** the configuration is changed and saved
- **THEN** "futureOption" is still in the file
