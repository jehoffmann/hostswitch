# SPDX-License-Identifier: Apache-2.0

Feature: Offline Devices (command line)

  @integration @spec-offline-devices @req-F-13a @bdd
  Scenario: Profile with an offline device
    Given the sample configuration
    When I run "hostswitchctl show"
    Then the exit status is 0
    And the output contains "Keyboard: Test Keyboard (ffff:ffff/1)"
