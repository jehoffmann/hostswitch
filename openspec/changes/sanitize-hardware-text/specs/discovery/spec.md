# Delta for discovery


## ADDED Requirements

### Requirement: SEC-11 Text from hardware is sanitised
The system SHALL remove control characters from every name or text read from hardware and SHALL limit its length before it is printed or stored.

#### Scenario: Escape sequence in a product name
- **GIVEN** a device reports a name containing an escape sequence and a newline
- **WHEN** devices are listed
- **THEN** the printed name contains no control characters and fits on one line
- **AND** the stored name is the sanitised one
