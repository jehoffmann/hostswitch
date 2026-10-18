# Delta for tray-app


# Tray App Specification

## Purpose
Switch hosts and edit the configuration without the command line.

## ADDED Requirements

### Requirement: F-01 Menu lists hosts and switches on selection
The system SHALL show a menu-bar item listing all configured hosts with the active host marked, and SHALL switch to a host when it is selected, showing the per-step result.

#### Scenario: Select a host
- **GIVEN** hosts "Mac" (active) and "PC"
- **WHEN** the user selects "PC"
- **THEN** the switch for "PC" runs and each step is shown as OK, FAIL or SKIP

#### Scenario: Switch in progress
- **WHEN** a switch is running
- **THEN** further selections are disabled until it finishes

### Requirement: F-02 Edit hosts and profiles
The system SHALL let the user create, edit and delete hosts and their profiles in a settings window, offering the detected displays and devices (also offline ones) as pick-lists and the standard input names, and SHALL reject invalid values with a readable message while keeping the previous configuration.

#### Scenario: Invalid change
- **GIVEN** the settings window is open
- **WHEN** the user enters a duplicate host label
- **THEN** a message explains the problem and the saved configuration is unchanged

### Requirement: F-09 Show which host owns which device
The system SHALL show in the menu which host each configured display and device currently belongs to, or "unknown".

#### Scenario: Mixed state
- **GIVEN** the display is on the Mac's input and the keyboard on the PC's slot
- **WHEN** the menu is opened
- **THEN** the display is shown as owned by "Mac" and the keyboard by "PC"

### Requirement: F-12 Hotkey per host
The system SHALL switch to a host when its configured global hotkey is pressed.

#### Scenario: Hotkey
- **GIVEN** the hotkey for "PC" is configured
- **WHEN** it is pressed
- **THEN** the same switch as selecting "PC" in the menu runs
