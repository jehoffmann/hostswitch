# Offline Devices Specification

## Purpose
Keyboards and mice that sleep or are connected to another computer are remembered, shown as offline and still usable in profiles.

## Requirements

### Requirement: F-13 Remember devices and show silent ones as offline
The system SHALL remember every device that answered during discovery and SHALL list remembered devices that did not answer as offline instead of dropping them.

#### Scenario: Device goes silent
- **GIVEN** a keyboard was seen online earlier and now does not answer
- **WHEN** the devices are listed
- **THEN** the keyboard is listed as Offline with its remembered name

#### Scenario: New device
- **GIVEN** a device that is not yet remembered answers
- **WHEN** the devices are listed
- **THEN** it is listed as Online and remembered

### Requirement: F-13a Profiles may use offline devices
The system SHALL accept profiles that reference remembered devices that are currently offline.

#### Scenario: Profile with an offline device
- **GIVEN** a profile references a device that is offline
- **WHEN** the configuration is loaded
- **THEN** it is accepted

### Requirement: F-13b Offline switching is retried once and reported
The system SHALL retry switching an offline device once after a short delay (it may be waking up) and, if it is still offline, SHALL report the step as failed with the reason "device offline".

#### Scenario: Device wakes up on retry
- **GIVEN** a keyboard that is offline on the first attempt and answers on the second
- **WHEN** the user switches to the host
- **THEN** the keyboard step succeeds after two attempts

#### Scenario: Device stays offline
- **GIVEN** a keyboard that stays offline
- **WHEN** the user switches to the host
- **THEN** the keyboard step is reported as FAIL with "device offline" after two attempts and the mouse is still switched

### Requirement: F-13c A device that left the computer is offline
The system SHALL treat a Bluetooth device that is no longer connected to this computer (it disappeared from the system) as offline, not as a tool failure.

#### Scenario: Keyboard moved to another host
- **GIVEN** a Bluetooth keyboard that switched to another computer and no longer exists on this one
- **WHEN** a switch is attempted on it
- **THEN** the step fails with "device offline"
