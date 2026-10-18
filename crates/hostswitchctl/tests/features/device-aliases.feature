# SPDX-License-Identifier: Apache-2.0

Feature: Device Aliases (command line)

  @integration @spec-device-aliases @req-R-2b @bdd
  Scenario: Link a second way
    Given the sample configuration
    When I run "hostswitchctl link ffff:ffff/1 ffff:ffff/9"
    Then the exit status is 0
    And the output contains "ffff:ffff/1 can now also be reached as ffff:ffff/9"
    When I run "hostswitchctl show"
    Then the output contains "(also via ffff:ffff/9)"
