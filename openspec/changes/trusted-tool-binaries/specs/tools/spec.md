# Delta for tools


## ADDED Requirements

### Requirement: SEC-5 Only trusted code gets device access
The system SHALL NOT hand device access to an executable that it did not verify: either hardware access happens in-process without a helper program, or a helper is started only from a configured absolute path whose content matches a recorded hash, with a minimal environment.

#### Scenario: Replaced binary
- **GIVEN** the recorded hash of a helper and a binary at its path that does not match
- **WHEN** the helper would be started
- **THEN** it is not started and the error names the mismatch

#### Scenario: Minimal environment
- **WHEN** a helper is started
- **THEN** it receives only the environment variables that it needs
