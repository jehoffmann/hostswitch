# ADR-0006: Configuration as a versioned, validated JSON file

- Status: Accepted
- Date: 2026-10-18

## Context
Hosts, profiles, remembered devices and aliases must survive restarts, be editable by hand and checkable. A broken file must not be silently replaced.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- spec:config/F-02a — location
- spec:config/F-02b — missing file
- spec:config/F-02c — versions and migration
- spec:config/F-02d — validation
- spec:config/F-02e — saving
- spec:config/F-02f — files from earlier versions
- spec:cli/F-07d — init builds a starting configuration
- change:fix-config-clobber/SEC-1 — never overwrite an unloadable file
- change:harden-config-storage/SEC-2 — private, symlink-safe saves
- change:validate-identifiers/SEC-3 — validated ids

## Decision
One JSON file: `$XDG_CONFIG_HOME/hostswitch/config.json`, else `~/.config/hostswitch/config.json` (macOS, Linux), `%APPDATA%\Hostswitch\config.json` (Windows), overridable with `--config`. Schema version 2 (`hosts`, `profiles`, `knownDevices`, optional `deviceAliases`); version 1 files migrate on load; other versions are rejected. Keys are camel case (`hostID`, `displayID`, `deviceID`). Validation runs on load and before every save; a save writes a temporary file and renames it. A missing file is an empty configuration. `init` builds a starting configuration with a pure function (`bootstrap`) from detected displays and online devices; the CLI prints it and writes only with `--write`.

## Consequences
- Human-readable, diffable, no extra dependency beyond `serde_json`.
- Saving and remembering devices are being hardened (changes `harden-config-storage` and `fix-config-clobber`); until then the file must not hold secrets.
- Fields this version does not know are not preserved on save (planned in `fix-config-clobber`).

## Alternatives considered
- TOML or YAML: friendlier to edit, but a new dependency and no benefit for nested lists of profiles.
- A platform settings database (UserDefaults, registry): not portable and not hand-editable.
