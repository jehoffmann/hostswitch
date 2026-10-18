# Delta for config


## ADDED Requirements

### Requirement: SEC-2 Private, symlink-safe, durable saves
The system SHALL create the configuration directory readable only by the user, SHALL write the configuration to a freshly created temporary file with a random name and permissions that allow only the user, SHALL flush it to disk and replace the old file atomically, and SHALL NOT follow a symbolic link at the temporary name. Concurrent saves SHALL NOT lose the data of either writer silently.

#### Scenario: Permissions
- **WHEN** the configuration is saved
- **THEN** the file is accessible only by its owner and so is its directory

#### Scenario: Symbolic link at the temporary name
- **GIVEN** a symbolic link at the former fixed temporary name pointing to another file
- **WHEN** the configuration is saved
- **THEN** the other file is not modified and the configuration is saved correctly

#### Scenario: Two writers
- **GIVEN** two processes save the configuration at the same time
- **WHEN** both finish
- **THEN** the file is valid JSON and one writer's complete configuration, never a mix
