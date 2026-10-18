# Delta for discovery


## ADDED Requirements

### Requirement: N-01 Displays are found on Intel Macs
The system SHALL list displays and read their current input on Intel Macs through a display backend suited to that hardware, chosen at runtime.

#### Scenario: Intel Mac
- **GIVEN** an Intel Mac with an external display
- **WHEN** the user lists displays
- **THEN** the display is listed with its current input
