# SPDX-License-Identifier: Apache-2.0

Feature: Switching (command line)
  End-to-end scenarios that run the real binary against a configuration with obviously fake ids, so that no hardware can be reached.

  @integration @spec-switching @req-F-01 @bdd
  Scenario: Unknown host or host without a profile
    Given the sample configuration
    When I run "hostswitchctl switch nobody"
    Then the exit status is 1
    And the error output contains "Unknown host: nobody"
    And no step result is printed
    When I run "hostswitchctl switch Bare"
    Then the exit status is 1
    And the error output contains "no profile for host 'Bare'"
    And no step result is printed
    And the configuration file is unchanged

  @integration @spec-switching @req-F-04 @bdd
  Scenario: Empty profile
    Given the sample configuration
    When I run "hostswitchctl switch Empty"
    Then the exit status is 0
    And the output contains "Nothing to switch"
    And no step result is printed
    And the configuration file is unchanged

  @integration @spec-switching @req-F-01b @bdd
  Scenario: Dry run
    Given the sample configuration
    When I run "hostswitchctl switch Work --dry-run"
    Then the exit status is 0
    And the output contains "Plan for 'Work':"
    And the output contains "1. Display:"
    And the output contains "2. Keyboard: Test Keyboard (ffff:ffff/1) -> Slot 1"
    And the output contains "3. Mouse:    Test Mouse (ffff:ffff/2) -> Slot 1"
    And the output contains "Dry run: nothing was changed."
    And no step result is printed
    When I run "hostswitchctl switch Work --dry"
    Then the exit status is 0
    And the configuration file is unchanged
