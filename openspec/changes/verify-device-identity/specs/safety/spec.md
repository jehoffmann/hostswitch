# Delta for safety


## ADDED Requirements

### Requirement: SEC-4a The device behind an id is the expected one
The system SHALL, before sending a set-current-host write to a receiver slot, check that the device answering there is the device that was remembered for that id, and SHALL refuse the write with a clear message if it is not.

#### Scenario: Slot re-paired
- **GIVEN** the remembered device for "046d:c52b/1" is a keyboard and the slot now answers as a mouse
- **WHEN** the keyboard is switched
- **THEN** no write is sent and the step fails with a message that the device changed

### Requirement: SEC-12 Replies are matched exactly
The system SHALL accept a reply or error report only if it answers the request just sent (device, feature, function and request marker).

#### Scenario: Unrelated error report
- **GIVEN** an error report for another feature arrives for the same device
- **WHEN** a request is waiting for its reply
- **THEN** the report is ignored
