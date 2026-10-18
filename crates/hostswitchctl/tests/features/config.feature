# SPDX-License-Identifier: Apache-2.0

Feature: Config (command line)

  @integration @spec-config @req-F-02b @bdd
  Scenario: First run
    Given no configuration file
    When I run "hostswitchctl show"
    Then the exit status is 0
    And the output contains "No hosts configured"
    And no configuration file exists
