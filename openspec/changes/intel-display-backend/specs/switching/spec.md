# Delta for switching


## ADDED Requirements

### Requirement: N-01a Display input is set on Intel Macs
The system SHALL set a display input on Intel Macs with the same plan, timeouts, retries and failure rules as on Apple Silicon.

#### Scenario: Intel switch
- **GIVEN** an Intel Mac and a profile with a display
- **WHEN** the user switches to that host
- **THEN** the display step is executed and reported like on Apple Silicon
