# SPDX-License-Identifier: Apache-2.0

Feature: Discovery
  Show which displays and which keyboards or mice are connected to this computer and how they are reachable.

  @unit @spec-discovery @req-F-05 @test:hostswitchctl/src/render.rs::display_list_aligns_and_handles_unknown_input_and_empty
  Scenario: One display
    Given a display "MSI MPG341CQR" on input 15
    When the user lists displays
    Then the output shows the name, its id and "Input: DP1 (15)"

  @unit @spec-discovery @req-F-05 @test:hostswitchctl/src/render.rs::display_list_aligns_and_handles_unknown_input_and_empty
  Scenario: Input cannot be read
    Given a display whose input cannot be read
    When the user lists displays
    Then the display is listed with "Input: unknown"

  @integration @spec-discovery @req-F-05a @test:hostswitch-backends/src/hidapitester.rs::discovery_finds_bluetooth_devices_with_names_and_kinds
  Scenario: Bluetooth devices
    Given a Bluetooth keyboard and a Bluetooth mouse that answer
    When the user lists devices
    Then both are listed as Online with their hardware names and the kind Keyboard or Mouse and the transport Bluetooth

  @integration @spec-discovery @req-F-05a @test:hostswitch-backends/src/hidapitester.rs::discovery_lists_only_genuine_ping_echoes
  Scenario: Receiver with one device and empty slots
    Given a receiver whose slot 1 answers and whose slots 2 to 6 are empty
    When the user lists devices
    Then exactly one receiver device is listed

  @integration @spec-discovery @req-F-05a @test:hostswitch-backends/src/hidapitester.rs::silence_is_retried_once_before_giving_up_on_a_slot
  Scenario: Silence on the first ping
    Given a sleeping receiver device that is silent on the first ping and answers on the second
    When the user lists devices
    Then the device is listed as Online

  @integration @spec-discovery @req-F-05b @test:hostswitch-backends/src/hidapitester.rs::discovery_finds_bluetooth_devices_with_names_and_kinds
  Scenario: Hardware reports "<manufacturer> - <product>"
    Given a Bluetooth device reported as "Vendor - ERGO K860" with keyboard capability
    When the user lists devices
    Then it is shown as "ERGO K860" with kind Keyboard

  @unit @spec-discovery @req-R-3a @test:hostswitchctl/src/render.rs::ports_list_marks_the_current_input
  Scenario: Current input marked
    Given a display on input 15
    When the user lists displays with ports
    Then "DP1 (15)" is marked as current and the note about the unreadable real ports is printed

  @unit @spec-discovery @req-R-3a @test:hostswitchctl/src/render.rs::ports_list_keeps_an_unnamed_current_input_and_handles_unknown
  Scenario: Vendor-specific current input
    Given a display on input 208
    When the user lists displays with ports
    Then "input 208" is listed and marked as current

  @integration @spec-discovery @req-R-5 @test:hostswitch-backends/src/hidapitester.rs::receiver_devices_survive_a_bluetooth_permission_denial
  Scenario: Only Bluetooth denied
    Given a receiver device answers and Bluetooth access is denied
    When the user lists devices
    Then the receiver device is listed

  @integration @spec-discovery @req-R-5 @test:hostswitch-backends/src/hidapitester.rs::input_monitoring_denial_is_reported_not_hidden
  Scenario: Everything denied
    Given Bluetooth access is denied and nothing else answers
    When the user lists devices
    Then an error mentions Input Monitoring and how to grant it

  @integration @spec-discovery @req-N-03a @test:hostswitch-backends/src/hidapitester.rs::reply_is_picked_out_of_unrelated_input_reports @test:hostswitch-backends/src/hidpp.rs::find_reply_skips_unrelated_input_reports
  Scenario: Reply behind mouse movement
    Given a moving mouse delivers two movement reports before the reply
    When a request is sent to it
    Then the reply is used

  @integration @spec-discovery @req-N-03a @test:hostswitch-backends/src/hidapitester.rs::only_noise_means_the_device_is_offline
  Scenario: Only noise
    Given only unrelated reports arrive
    When a request is sent
    Then the device is treated as offline and no write is sent
