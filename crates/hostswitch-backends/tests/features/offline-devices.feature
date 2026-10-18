# SPDX-License-Identifier: Apache-2.0

Feature: Offline Devices (backend behaviour)
  A Bluetooth device that left this computer disappears from the system; that must read as "offline", not as a tool failure.

  @integration @spec-offline-devices @req-F-13c @test:hostswitch-backends/src/hidapitester.rs::device_missing_from_the_system_is_offline_not_a_tool_failure
  Scenario: Keyboard moved to another host
    Given a Bluetooth keyboard that switched to another computer and no longer exists on this one
    When a switch is attempted on it
    Then the step fails with "device offline"
