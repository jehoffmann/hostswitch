# SPDX-License-Identifier: Apache-2.0

Feature: Device Aliases
  One physical keyboard or mouse can be reachable in different ways (Bluetooth in one Easy-Switch slot, a Unifying receiver in another). Aliases let one profile work in both directions.

  @integration @spec-device-aliases @req-R-2a @test:hostswitch-core/tests/executor.rs::offline_primary_falls_back_to_the_alias
  Scenario: Primary offline, alias works
    Given keyboard "kb" has alias "kb-usb" and "kb" is offline
    When the user switches to a host using "kb"
    Then the switch is sent to "kb-usb" and succeeds

  @integration @spec-device-aliases @req-R-2a @test:hostswitch-core/tests/executor.rs::working_primary_never_touches_aliases
  Scenario: Primary works
    Given keyboard "kb" with an alias
    When the primary id answers
    Then the alias is never used

  @integration @spec-device-aliases @req-R-2a @test:hostswitch-core/tests/executor.rs::real_errors_are_not_masked_by_aliases
  Scenario: Real error is not masked
    Given the primary id fails with an error other than offline
    When the step runs
    Then the step fails with that error and no alias is tried

  @integration @spec-device-aliases @req-R-2a @test:hostswitch-core/tests/executor.rs::all_ways_offline_is_reported_as_device_offline
  Scenario: Every way is offline
    Given the primary id and all aliases are offline
    When the step runs
    Then the step is reported as FAIL with "device offline"

  @unit @spec-device-aliases @req-R-2b @test:hostswitch-core/src/config.rs::conflicting_aliases_are_rejected_and_leave_the_config_unchanged
  Scenario: Conflicting alias
    Given "a" already has alias "b"
    When the user links "c" with "b"
    Then the request is rejected and the configuration is unchanged

  @gap @system @spec-device-aliases @req-R-2c
  Scenario: Status while the device is on the receiver
    Given the keyboard only answers through its alias
    When the status is shown
    Then the keyboard's current slot is shown
