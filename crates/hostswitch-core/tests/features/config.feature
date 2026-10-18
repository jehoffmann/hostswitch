# SPDX-License-Identifier: Apache-2.0

Feature: Config
  Hosts, profiles, remembered devices and aliases are stored in one JSON file that is validated, never silently corrupted and readable by humans.

  @unit @spec-config @req-F-02a @test:hostswitch-core/src/config.rs::default_path_follows_xdg_with_home_fallback
  Scenario: Default and override
    Given XDG_CONFIG_HOME is unset and HOME is "/home/u"
    When the default path is resolved
    Then it is "/home/u/.config/hostswitch/config.json"

  @unit @spec-config @req-F-02a @test:hostswitch-core/src/config.rs::default_path_follows_xdg_with_home_fallback
  Scenario: Relative XDG value
    Given XDG_CONFIG_HOME is "relative"
    When the default path is resolved
    Then the value is ignored and the HOME based path is used

  @unit @spec-config @req-F-02c @test:hostswitch-core/src/config.rs::v1_migrates_to_v2_and_other_versions_are_rejected
  Scenario: Version 1 file
    Given a version 1 file with hosts and profiles
    When it is loaded
    Then it loads as version 2 with an empty list of remembered devices

  @unit @spec-config @req-F-02c @test:hostswitch-core/src/config.rs::v1_migrates_to_v2_and_other_versions_are_rejected
  Scenario: Version 3 file
    Given a file with version 3
    When it is loaded
    Then it is rejected with an "unsupported configuration version" error

  @unit @spec-config @req-F-02d @test:hostswitch-core/src/config.rs::validation_rules
  Scenario: Duplicate label
    Given two hosts labelled "A" and "a"
    When the configuration is validated
    Then it is rejected

  @unit @spec-config @req-F-02d @test:hostswitch-core/src/config.rs::rejects_out_of_range_values
  Scenario: Slot out of range
    Given a profile with keyboard slot 4
    When the configuration is loaded
    Then it is rejected with a readable error

  @unit @spec-config @req-F-02d @test:hostswitch-core/src/config.rs::corrupt_input_is_a_readable_error
  Scenario: Malformed input
    Given an empty, truncated or wrongly typed JSON file
    When the configuration is loaded
    Then a readable "invalid configuration" error is reported

  @unit @spec-config @req-F-02e @test:hostswitch-core/tests/config_store.rs::save_is_atomic_and_keeps_old_file_on_rejection
  Scenario: Rejected save
    Given a valid configuration on disk
    When an invalid configuration is saved
    Then the save fails and the file on disk is unchanged

  @unit @spec-config @req-F-02f @test:hostswitch-core/src/config.rs::file_from_an_earlier_version_decodes
  Scenario: File written by an earlier version
    Given a file as written by an earlier version
    When it is loaded
    Then hosts, profiles, devices and slots are read correctly
