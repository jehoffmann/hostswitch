# Delta for config


## ADDED Requirements

### Requirement: SEC-3 Display ids are validated
The system SHALL accept as a display id only the stable system identifier format that the display tool reports, SHALL reject every other value (including words, numbers and values starting with "-") when the configuration is loaded, and SHALL NOT pass an unvalidated id to a tool.

#### Scenario: Id that addresses nothing
- **GIVEN** a profile with display id "list"
- **WHEN** the configuration is loaded
- **THEN** it is rejected with an error naming the display id

#### Scenario: Id that looks like an option
- **GIVEN** a display id "-1"
- **WHEN** the configuration is loaded
- **THEN** it is rejected

### Requirement: SEC-4 Device ids are canonical
The system SHALL accept device ids only as vendor id, product id and index in lower-case hexadecimal form with an index of 1 to 6 or 255, SHALL compare ids in that canonical form, and SHALL reject an alias that is the same device as another profile device, and a keyboard and mouse that are the same device, however they are written.

#### Scenario: Differently written id
- **GIVEN** keyboard "046d:c52b/1" and mouse "046D:C52B/01"
- **WHEN** the configuration is loaded
- **THEN** it is rejected because both are the same device

#### Scenario: Malformed id
- **GIVEN** a device id "0046d:c52b/2"
- **WHEN** the configuration is loaded
- **THEN** it is rejected
