# SPDX-License-Identifier: Apache-2.0

Feature: Offline Devices
  Keyboards and mice that sleep or are connected to another computer are remembered, shown as offline and still usable in profiles.

  @unit @spec-offline-devices @req-F-13 @test:hostswitch-core/src/domain.rs::merge_puts_live_first_and_marks_silent_known_offline
  Scenario: Device goes silent
    Given a keyboard was seen online earlier and now does not answer
    When the devices are listed
    Then the keyboard is listed as Offline with its remembered name

  @unit @spec-offline-devices @req-F-13 @test:hostswitch-core/src/config.rs::remember_adds_updates_and_ignores_offline
  Scenario: New device
    Given a device that is not yet remembered answers
    When the devices are listed
    Then it is listed as Online and remembered

  @integration @spec-offline-devices @req-F-13b @test:hostswitch-core/tests/executor.rs::offline_device_waking_on_retry_succeeds
  Scenario: Device wakes up on retry
    Given a keyboard that is offline on the first attempt and answers on the second
    When the user switches to the host
    Then the keyboard step succeeds after two attempts

  @integration @spec-offline-devices @req-F-13b @test:hostswitch-core/tests/executor.rs::offline_device_is_retried_once_then_reported_offline
  Scenario: Device stays offline
    Given a keyboard that stays offline
    When the user switches to the host
    Then the keyboard step is reported as FAIL with "device offline" after two attempts and the mouse is still switched
