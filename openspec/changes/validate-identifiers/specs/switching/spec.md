# Delta for switching


## ADDED Requirements

### Requirement: SEC-13 The dry run shows every possible target
The system SHALL show in the dry-run plan, for each keyboard and mouse step, the id from the profile and every alias that may receive the write if the first is offline.

#### Scenario: Keyboard with an alias
- **GIVEN** the keyboard has an alias
- **WHEN** the user switches with the dry-run option
- **THEN** the plan line names the alias as the fallback
