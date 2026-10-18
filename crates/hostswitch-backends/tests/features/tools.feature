# SPDX-License-Identifier: Apache-2.0

Feature: Tools
  External helper programs (m1ddc, hidapitester) are found, run with a timeout and their output is parsed in isolation.

  @unit @spec-tools @req-N-03 @test:hostswitch-backends/tests/tool_runner.rs::missing_tool_is_backend_unavailable
  Scenario: Tool is missing
    Given a tool that is not installed
    When it is needed
    Then a "backend unavailable" error names the tool

  @unit @spec-tools @req-N-03 @test:hostswitch-backends/tests/tool_runner.rs::large_output_on_both_pipes_does_not_deadlock
  Scenario: Large output
    Given a tool that writes 300 KB to both outputs
    When it is run
    Then all output is captured and the call returns

  @unit @spec-tools @req-N-03 @test:hostswitch-backends/tests/tool_runner.rs::non_utf8_output_is_decoded_lossily
  Scenario: Non-UTF-8 output
    Given a tool that prints a byte that is not valid UTF-8
    When it is run
    Then the output is decoded leniently and the call succeeds

  @unit @spec-tools @req-N-03b @test:hostswitch-backends/src/m1ddc.rs::empty_first_read_is_retried_and_never_means_no_input
  Scenario: Empty first answer
    Given the tool answers empty once and then "15"
    When the input is read
    Then the input 15 is returned after two attempts

  @unit @spec-tools @req-N-03c @test:hostswitch-backends/src/m1ddc.rs::list_parser_tolerates_garbage
  Scenario: Garbage list
    Given garbage instead of a display list
    When it is parsed
    Then no displays are returned
