# SPDX-License-Identifier: Apache-2.0

Feature: CLI
  hostswitchctl exposes every operation for scripting and humans with a consistent, vendor-neutral output.

  @integration @spec-cli @req-F-07 @bdd
  Scenario: Help
    When I run "hostswitchctl --help"
    Then the exit status is 0
    And the output contains "list"
    And the output contains "show"
    And the output contains "status"
    And the output contains "init"
    And the output contains "link"
    And the output contains "switch"

  @integration @spec-cli @req-F-07a @bdd
  Scenario: Unknown host
    Given the sample configuration
    When I run "hostswitchctl switch nobody --dry-run"
    Then the exit status is 1
    And the error output contains "Unknown host: nobody"

  @integration @spec-cli @req-F-07a @test:hostswitchctl/src/args.rs::rejects_unknown_input @bdd
  Scenario: Missing argument or unknown command
    When I run "hostswitchctl switch"
    Then the exit status is 64
    When I run "hostswitchctl frobnicate"
    Then the exit status is 64

  @integration @spec-cli @req-F-07b @test:hostswitchctl/src/render.rs::show_uses_capitalised_labels_and_aligns_values @bdd
  Scenario: Two hosts
    Given the sample configuration
    When I run "hostswitchctl show"
    Then the exit status is 0
    And the output contains "Work  (hostname mbp.local)"
    And the output contains "Display:  00000000-0000-4000-8000-0000000000d1"
    And the output contains "Keyboard: Test Keyboard (ffff:ffff/1)"
    And the output contains "Mouse:    Test Mouse (ffff:ffff/2)"
    And the output contains "(no profile)"
    And the output contains "(empty profile)"
    And the configuration file is unchanged

  @integration @spec-cli @req-F-07b @bdd
  Scenario: No hosts
    Given an empty configuration
    When I run "hostswitchctl show"
    Then the exit status is 0
    And the output contains "hostswitchctl init"

  @unit @spec-cli @req-F-07c @test:hostswitchctl/src/render.rs::status_groups_by_host_and_compares_with_the_hardware
  Scenario: Mac active
    Given the display and devices are in the state of host "Mac"
    When the user runs status
    Then "Mac" is marked with * and its values are OK
    And the values of "PC" show the current state, for example "now: Slot 1"

  @system @manual @spec-cli @req-F-07d
  Scenario: Print only
    When the user runs init without --write
    Then the configuration is printed and the file is not changed

  @integration @spec-cli @req-F-07d @bdd
  Scenario: Existing hosts are protected
    Given the sample configuration
    When I run "hostswitchctl init --write"
    Then the exit status is 1
    And the error output contains "already has hosts"
    And the configuration file is unchanged

  @unit @spec-cli @req-F-07d @test:hostswitch-core/src/bootstrap.rs::too_many_hosts_and_duplicate_labels_are_rejected
  Scenario: Too many hosts
    When the user passes three --host options
    Then the command fails because Easy-Switch has three slots

  @integration @spec-cli @req-F-07d @test:hostswitchctl/src/args.rs::init_rejects_bad_input_and_force_without_write @bdd
  Scenario: Unknown input name
    When I run "hostswitchctl init --host PC=dp9"
    Then the exit status is 64
    And the error output contains "unknown display input 'dp9'"

  @integration @spec-cli @req-F-07e @bdd
  Scenario: Link
    Given the sample configuration
    When I run "hostswitchctl link ffff:ffff/1 ffff:ffff/9"
    Then the exit status is 0
    And the output contains "can now also be reached as"
    And the configuration file contains "ffff:ffff/9"

  @unit @spec-cli @req-F-07f @test:hostswitchctl/src/render.rs::plan_uses_names_and_capitalised_roles @test:hostswitchctl/src/render.rs::report_lines_show_ok_fail_skip
  Scenario: Plan and report lines
    Given a switch with a display, a keyboard and a mouse
    When the plan and the report are printed
    Then lines look like "1. Display:  <id> -> DP1 (15)" and "OK    Keyboard: ERGO K860 (<id>) -> Slot 2"
