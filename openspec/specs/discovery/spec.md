# Discovery Specification

## Purpose
Show which displays and which keyboards or mice are connected to this computer and how they are reachable.

## Requirements

### Requirement: F-05 List displays with their current input
The system SHALL list every detected display with its name, id and current input, showing "unknown" when the input cannot be read.

#### Scenario: One display
- **GIVEN** a display "MSI MPG341CQR" on input 15
- **WHEN** the user lists displays
- **THEN** the output shows the name, its id and "Input: DP1 (15)"

#### Scenario: Input cannot be read
- **GIVEN** a display whose input cannot be read
- **WHEN** the user lists displays
- **THEN** the display is listed with "Input: unknown"

### Requirement: F-05a List keyboards and mice from receivers and Bluetooth
The system SHALL list devices paired with a USB receiver and devices connected over Bluetooth. A device SHALL be listed as online only if it answered a genuine ping; an empty receiver slot SHALL NOT be listed as a device.

#### Scenario: Bluetooth devices
- **GIVEN** a Bluetooth keyboard and a Bluetooth mouse that answer
- **WHEN** the user lists devices
- **THEN** both are listed as Online with their hardware names and the kind Keyboard or Mouse and the transport Bluetooth

#### Scenario: Receiver with one device and empty slots
- **GIVEN** a receiver whose slot 1 answers and whose slots 2 to 6 are empty
- **WHEN** the user lists devices
- **THEN** exactly one receiver device is listed

#### Scenario: Silence on the first ping
- **GIVEN** a sleeping receiver device that is silent on the first ping and answers on the second
- **WHEN** the user lists devices
- **THEN** the device is listed as Online

### Requirement: F-05b Names and kinds come from the hardware
The system SHALL show the product name reported by the hardware without a manufacturer prefix, SHALL derive keyboard or mouse from the reported device capabilities, and SHALL NOT contain a hard-coded manufacturer name in its own output. Receiver devices without a readable name SHALL be called "Unifying device N" or "Bolt device N".

#### Scenario: Hardware reports "<manufacturer> - <product>"
- **GIVEN** a Bluetooth device reported as "Vendor - ERGO K860" with keyboard capability
- **WHEN** the user lists devices
- **THEN** it is shown as "ERGO K860" with kind Keyboard

### Requirement: R-3a List the standard display input ports
The system SHALL, on request, list the standard display input values (VGA1/2, DVI1/2, DP1/2, HDMI1/2, USB-C) per display, mark the current one with `*`, add a current value without a standard name to the list, and state that the real ports of the monitor cannot be read.

#### Scenario: Current input marked
- **GIVEN** a display on input 15
- **WHEN** the user lists displays with ports
- **THEN** "DP1 (15)" is marked as current and the note about the unreadable real ports is printed

#### Scenario: Vendor-specific current input
- **GIVEN** a display on input 208
- **WHEN** the user lists displays with ports
- **THEN** "input 208" is listed and marked as current

### Requirement: R-5 Permission problems are explained
The system SHALL report a macOS Input Monitoring denial with instructions when it prevents reading any device, and SHALL still list receiver devices when only Bluetooth access is denied.

#### Scenario: Only Bluetooth denied
- **GIVEN** a receiver device answers and Bluetooth access is denied
- **WHEN** the user lists devices
- **THEN** the receiver device is listed

#### Scenario: Everything denied
- **GIVEN** Bluetooth access is denied and nothing else answers
- **WHEN** the user lists devices
- **THEN** an error mentions Input Monitoring and how to grant it

### Requirement: N-03a Replies are matched to requests
The system SHALL ignore input reports that do not answer its request (mouse movement, key events, notifications) and use only a reply for the same device, feature and function, or an error report for the device.

#### Scenario: Reply behind mouse movement
- **GIVEN** a moving mouse delivers two movement reports before the reply
- **WHEN** a request is sent to it
- **THEN** the reply is used

#### Scenario: Only noise
- **GIVEN** only unrelated reports arrive
- **WHEN** a request is sent
- **THEN** the device is treated as offline and no write is sent
