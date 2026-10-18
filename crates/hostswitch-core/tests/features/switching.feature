# SPDX-License-Identifier: Apache-2.0

Feature: Switching
  Move the desk to a host: set the display inputs, then the keyboard and the mouse slot named in that host's profile, safely and with a per-step result.

  @integration @spec-switching @req-F-01 @test:hostswitch-core/tests/executor.rs::happy_path_runs_in_order_across_backends
  Scenario: Full profile is applied
    Given host "Work" has a profile with one display, a keyboard and a mouse
    When the user switches to "Work"
    Then the display input is set, the keyboard is switched to its slot and the mouse is switched to its slot
    And each step is reported as OK

  @unit @spec-switching @req-F-01 @test:hostswitch-core/src/config.rs::host_matching_prefers_label_then_hostname_then_uuid
  Scenario: Host is found by label, hostname or id
    Given hosts "Work" with hostname "mbp.local" and "Home"
    When the user names a host as "WORK", "mbp.local" or its id
    Then the host "Work" is selected

  @integration @spec-switching @req-F-01a @test:hostswitch-core/tests/executor.rs::happy_path_runs_in_order_across_backends @test:hostswitch-core/src/planner.rs::orders_displays_then_keyboard_then_mouse
  Scenario: Cross-backend order
    Given a profile with two displays, a keyboard and a mouse
    When the plan is executed
    Then both displays are set before the keyboard, and the keyboard before the mouse

  @unit @spec-switching @req-F-03 @test:hostswitch-core/src/planner.rs::five_displays_plan_five_display_steps
  Scenario: Five displays
    Given a profile with five displays
    When the user switches to that host
    Then five display steps are planned, one per display

  @unit @spec-switching @req-F-04 @test:hostswitch-core/src/planner.rs::optional_parts_and_empty_profile
  Scenario: Mouse missing
    Given a profile with a display and a keyboard but no mouse
    When the user switches to that host
    Then only the display and the keyboard are switched

  @integration @spec-switching @req-N-04a @test:hostswitch-core/tests/executor.rs::failed_display_skips_keyboard_and_mouse
  Scenario: Display fails
    Given a profile with a display, a keyboard and a mouse, and the display cannot be set
    When the user switches to that host
    Then the display step is reported as FAIL
    And the keyboard and mouse steps are reported as SKIP and no command is sent to them

  @integration @spec-switching @req-N-04a @test:hostswitch-core/tests/executor.rs::keyboard_failure_does_not_skip_mouse
  Scenario: Keyboard fails
    Given the keyboard cannot be switched
    When the user switches to the host
    Then the keyboard step is reported as FAIL and the mouse is still switched

  @integration @spec-switching @req-N-04a @test:hostswitch-core/tests/executor.rs::other_displays_still_run_after_a_display_failure
  Scenario: One of two displays fails
    Given two displays of which the first cannot be set
    When the user switches to the host
    Then the second display is still set and the keyboard and mouse are skipped

  @integration @spec-switching @req-N-04b @test:hostswitch-core/tests/executor.rs::blocking_backend_cannot_delay_the_caller_past_the_timeout @test:hostswitch-core/tests/executor.rs::cancellable_slow_step_times_out_and_skips_peripherals
  Scenario: Step that never finishes
    Given a display backend that blocks for longer than the step timeout
    When the step is executed
    Then the step is reported as failed with a timeout message within the timeout, not when the backend finally returns

  @integration @spec-switching @req-N-04b @test:hostswitch-core/tests/executor.rs::transient_failure_is_retried
  Scenario: Transient failure
    Given a display that fails once and then succeeds
    When the step is executed
    Then the step succeeds on the second attempt

  @integration @spec-switching @req-N-04c @test:hostswitch-core/tests/executor.rs::missing_backend_fails_without_retry
  Scenario: No display tool installed
    Given no display backend is available
    When the user switches to a host with a display
    Then the display step is reported as failed with a "backend unavailable" reason after one attempt

  @gap @integration @spec-switching @req-F-01c
  Scenario: Partial failure
    Given a switch in which one step fails
    When the switch finishes
    Then the exit status is 2
