# SPDX-License-Identifier: Apache-2.0

Feature: Safety
  hostswitch only switches channels. It must never send a command that could damage or disable a monitor or peripheral.

  @integration @spec-safety @req-S-01 @test:hostswitch-backends/src/hidapitester.rs::bluetooth_switch_sends_a_long_report_to_index_ff_after_lookup @test:hostswitch-backends/src/hidapitester.rs::switch_sends_the_write_only_after_a_successful_feature_lookup
  Scenario: Switching a keyboard
    When a keyboard is switched
    Then the only write sent to it is the set-current-host request, with the slot of the profile

  @integration @spec-safety @req-S-02 @test:hostswitch-backends/src/hidapitester.rs::switch_never_writes_when_the_lookup_fails
  Scenario: Lookup fails
    Given the feature lookup returns an error, silence, garbage or "unsupported"
    When a keyboard switch is attempted
    Then no set-current-host request is sent

  @integration @spec-safety @req-S-03 @test:hostswitch-backends/src/hidapitester.rs::current_host_never_sends_a_write_frame
  Scenario: Status
    When the current host of a device is read
    Then only read requests are sent

  @unit @spec-safety @req-S-04 @test:hostswitch-backends/tests/tool_runner.rs::arguments_are_not_interpreted_by_a_shell
  Scenario: Hostile characters in an argument
    Given an argument containing shell metacharacters
    When a tool is started
    Then the tool receives the argument verbatim

  @unit @spec-safety @req-S-04 @test:hostswitch-backends/tests/tool_runner.rs::timeout_kills_the_process
  Scenario: Tool hangs
    Given a tool that does not exit
    When its timeout elapses
    Then it is killed and a timeout error is reported

  @gap @integration @spec-safety @req-S-05
  Scenario: Dry run
    When a switch is requested with the dry-run option
    Then no display or device backend is used
