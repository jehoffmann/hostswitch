// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Host, KnownDevice, Peripheral, Profile};
use crate::error::{HostswitchError, Result};

pub const CURRENT_VERSION: i64 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "RawConfig")]
pub struct AppConfig {
    pub version: i64,
    pub hosts: Vec<Host>,
    pub profiles: Vec<Profile>,
    /// Devices seen by discovery, kept so they can be shown offline and used in profiles (F-13). Added in v2.
    #[serde(rename = "knownDevices")]
    pub known_devices: Vec<KnownDevice>,
    /// Other ways to reach the same physical device, tried in order when the primary id is offline.
    /// Example: the keyboard over Bluetooth (`046d:b359/255`) is also reachable through the Unifying
    /// receiver (`046d:c52b/1`) while it is connected to this computer's receiver.
    #[serde(
        default,
        rename = "deviceAliases",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub device_aliases: BTreeMap<String, Vec<String>>,
}

/// Wire shape. Migration: v1 files have no `knownDevices`; they load as v2 with an empty list. Versions
/// outside 1..=CURRENT_VERSION are kept as-is so `validate()` rejects them. A missing `version` is a decode error.
#[derive(Deserialize)]
struct RawConfig {
    version: i64,
    hosts: Vec<Host>,
    profiles: Vec<Profile>,
    #[serde(default, rename = "knownDevices")]
    known_devices: Vec<KnownDevice>,
    #[serde(default, rename = "deviceAliases")]
    device_aliases: BTreeMap<String, Vec<String>>,
}

impl From<RawConfig> for AppConfig {
    fn from(raw: RawConfig) -> Self {
        Self {
            version: if raw.version == 1 {
                CURRENT_VERSION
            } else {
                raw.version
            },
            hosts: raw.hosts,
            profiles: raw.profiles,
            known_devices: raw.known_devices,
            device_aliases: raw.device_aliases,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            hosts: Vec::new(),
            profiles: Vec::new(),
            known_devices: Vec::new(),
            device_aliases: BTreeMap::new(),
        }
    }
}

impl AppConfig {
    /// Adds or refreshes remembered devices from online ones. Returns true if anything changed.
    pub fn remember(&mut self, devices: &[Peripheral]) -> bool {
        let mut changed = false;
        for device in devices.iter().filter(|d| d.is_online()) {
            let known = KnownDevice::from(device);
            match self.known_devices.iter_mut().find(|k| k.id == device.id) {
                Some(existing) if *existing != known => {
                    *existing = known;
                    changed = true;
                }
                Some(_) => {}
                None => {
                    self.known_devices.push(known);
                    changed = true;
                }
            }
        }
        changed
    }

    /// The device id followed by its aliases: every way to reach the device, in the order to try them.
    pub fn device_candidates(&self, id: &str) -> Vec<String> {
        let mut ids = vec![id.to_string()];
        ids.extend(self.device_aliases.get(id).into_iter().flatten().cloned());
        ids
    }

    /// Declares `also` as another way to reach `primary`. Validates before returning.
    pub fn link_device(&mut self, primary: &str, also: &str) -> Result<()> {
        let mut candidate = self.clone();
        let list = candidate
            .device_aliases
            .entry(primary.to_string())
            .or_default();
        if !list.iter().any(|a| a == also) {
            list.push(also.to_string());
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    pub fn profile(&self, host_id: Uuid) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.host_id == host_id)
    }

    /// Matches by label or hostname (case-insensitive), then by UUID string.
    pub fn host_matching(&self, query: &str) -> Option<&Host> {
        let q = query.to_lowercase();
        self.hosts
            .iter()
            .find(|h| h.label.to_lowercase() == q)
            .or_else(|| self.hosts.iter().find(|h| h.hostname.to_lowercase() == q))
            .or_else(|| {
                self.hosts
                    .iter()
                    .find(|h| h.id.to_string().to_lowercase() == q)
            })
    }

    pub fn validate(&self) -> Result<()> {
        let invalid = |m: &str| Err(HostswitchError::InvalidConfig(m.to_string()));
        if !(1..=CURRENT_VERSION).contains(&self.version) {
            return Err(HostswitchError::UnsupportedConfigVersion(self.version));
        }
        if !all_unique(self.hosts.iter().map(|h| h.id)) {
            return invalid("duplicate host id");
        }
        if !all_unique(self.hosts.iter().map(|h| h.label.to_lowercase())) {
            return invalid("duplicate host label");
        }
        let hostnames = self
            .hosts
            .iter()
            .map(|h| h.hostname.trim().to_lowercase())
            .filter(|h| !h.is_empty());
        if !all_unique(hostnames) {
            return invalid("duplicate hostname");
        }
        if !all_unique(self.known_devices.iter().map(|k| k.id.as_str())) {
            return invalid("duplicate known device");
        }
        let mut alias_ids: HashSet<&str> = HashSet::new();
        for (primary, aliases) in &self.device_aliases {
            if aliases.is_empty() || primary.trim().is_empty() {
                return invalid("device alias without a device or without targets");
            }
            for alias in aliases {
                let clash = alias == primary
                    || alias.trim().is_empty()
                    || self.device_aliases.contains_key(alias)
                    || !alias_ids.insert(alias.as_str());
                if clash {
                    return invalid(&format!(
                        "device alias '{alias}' conflicts with another device id"
                    ));
                }
            }
        }
        if self.hosts.iter().any(|h| h.label.trim().is_empty()) {
            return invalid("empty host label");
        }
        let mut seen = HashSet::new();
        for profile in &self.profiles {
            if !self.hosts.iter().any(|h| h.id == profile.host_id) {
                return invalid(&format!(
                    "profile references unknown host {}",
                    profile.host_id
                ));
            }
            if !seen.insert(profile.host_id) {
                return invalid(&format!("multiple profiles for host {}", profile.host_id));
            }
            if !all_unique(profile.displays.iter().map(|d| d.display_id.as_str())) {
                return invalid("profile lists a display twice");
            }
            if let (Some(kb), Some(mouse)) = (&profile.keyboard, &profile.mouse)
                && kb.device_id == mouse.device_id
            {
                return invalid("keyboard and mouse use the same device");
            }
        }
        Ok(())
    }
}

fn all_unique<T: std::hash::Hash + Eq>(items: impl Iterator<Item = T>) -> bool {
    let mut seen = HashSet::new();
    items.into_iter().all(|i| seen.insert(i))
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    pub path: PathBuf,
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self {
            path: Self::default_path(),
        }
    }
}

impl ConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// `$XDG_CONFIG_HOME/hostswitch/config.json`, else `~/.config/hostswitch/config.json` (macOS and Linux),
    /// `%APPDATA%\\Hostswitch\\config.json` on Windows.
    pub fn default_path() -> PathBuf {
        let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
        Self::resolve_path(
            cfg!(target_os = "windows"),
            env("XDG_CONFIG_HOME"),
            env("APPDATA"),
            env("HOME").unwrap_or_default(),
        )
    }

    fn resolve_path(
        windows: bool,
        xdg: Option<PathBuf>,
        appdata: Option<PathBuf>,
        home: PathBuf,
    ) -> PathBuf {
        if windows {
            return appdata
                .unwrap_or(home)
                .join("Hostswitch")
                .join("config.json");
        }
        // The XDG spec says relative values must be ignored.
        let base = xdg
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home.join(".config"));
        base.join("hostswitch").join("config.json")
    }

    /// A missing file yields an empty default config rather than an error.
    pub fn load(&self) -> Result<AppConfig> {
        match std::fs::read(&self.path) {
            Ok(data) => Self::decode(&data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(e) => Err(HostswitchError::InvalidConfig(format!(
                "cannot read {}: {e}",
                self.path.display()
            ))),
        }
    }

    /// Validates, then writes atomically (temp file + rename) so a failure keeps the previous file.
    pub fn save(&self, config: &AppConfig) -> Result<()> {
        config.validate()?;
        let data = Self::encode(config)?;
        let io = |e: std::io::Error| {
            HostswitchError::InvalidConfig(format!("cannot write {}: {e}", self.path.display()))
        };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let tmp = tmp_path(&self.path);
        std::fs::write(&tmp, data).map_err(io)?;
        std::fs::rename(&tmp, &self.path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            io(e)
        })
    }

    pub fn decode(data: &[u8]) -> Result<AppConfig> {
        let config: AppConfig = serde_json::from_slice(data)
            .map_err(|e| HostswitchError::InvalidConfig(format!("malformed JSON ({e})")))?;
        config.validate()?;
        Ok(config)
    }

    pub fn encode(config: &AppConfig) -> Result<Vec<u8>> {
        serde_json::to_vec_pretty(config).map_err(|e| HostswitchError::InvalidConfig(e.to_string()))
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;

    fn host(label: &str, hostname: &str) -> Host {
        Host::new(label, hostname)
    }

    fn with_profile(h: &Host) -> AppConfig {
        let mut p = Profile::new(h.id);
        p.displays.push(DisplayAssignment {
            display_id: "D1".into(),
            input: DisplayInput(15),
        });
        p.keyboard = Some(DeviceAssignment {
            device_id: "046d:c52b/1".into(),
            slot: Slot::new(1).unwrap(),
        });
        AppConfig {
            hosts: vec![h.clone()],
            profiles: vec![p],
            ..AppConfig::default()
        }
    }

    #[test]
    fn round_trips_through_json() {
        let config = with_profile(&host("Mac", "mac.local"));
        let data = ConfigStore::encode(&config).unwrap();
        assert_eq!(ConfigStore::decode(&data).unwrap(), config);
    }

    #[test]
    fn file_from_an_earlier_version_decodes() {
        let json = r#"{"version":2,"hosts":[{"id":"1C95A13D-CA9A-4160-A4BD-E0DFA6B99D52","label":"Mac","hostname":"mac"}],
      "profiles":[{"hostID":"1C95A13D-CA9A-4160-A4BD-E0DFA6B99D52","displays":[{"displayID":"X","input":17}],
      "keyboard":{"deviceID":"046d:c52b/1","slot":2}}],
      "knownDevices":[{"id":"046d:c52b/1","name":"K","kind":"keyboard","transport":"unifying"}]}"#;
        let c = ConfigStore::decode(json.as_bytes()).unwrap();
        assert_eq!(c.profiles[0].keyboard.as_ref().unwrap().slot.get(), 2);
        assert_eq!(c.known_devices[0].kind, PeripheralKind::Keyboard);
        assert!(c.profiles[0].mouse.is_none());
    }

    #[test]
    fn v1_migrates_to_v2_and_other_versions_are_rejected() {
        let v1 = br#"{"version":1,"hosts":[],"profiles":[]}"#;
        let c = ConfigStore::decode(v1).unwrap();
        assert_eq!(c.version, 2);
        assert!(c.known_devices.is_empty());
        for v in [0, 3] {
            let json = format!(r#"{{"version":{v},"hosts":[],"profiles":[]}}"#);
            assert_eq!(
                ConfigStore::decode(json.as_bytes()),
                Err(HostswitchError::UnsupportedConfigVersion(v))
            );
        }
    }

    #[test]
    fn corrupt_input_is_a_readable_error() {
        for bad in [
            &b""[..],
            b"{",
            b"[]",
            br#"{"hosts":[],"profiles":[]}"#,
            br#"{"version":"2","hosts":[],"profiles":[]}"#,
        ] {
            let err = ConfigStore::decode(bad).unwrap_err();
            assert!(matches!(err, HostswitchError::InvalidConfig(_)), "{err:?}");
        }
    }

    #[test]
    fn rejects_out_of_range_values() {
        let h = host("Mac", "mac");
        let id = h.id;
        let slot = format!(
            r#"{{"version":2,"hosts":[{}],"profiles":[{{"hostID":"{id}","keyboard":{{"deviceID":"k","slot":4}}}}]}}"#,
            serde_json::to_string(&h).unwrap()
        );
        assert!(ConfigStore::decode(slot.as_bytes()).is_err());
        let input = format!(
            r#"{{"version":2,"hosts":[{}],"profiles":[{{"hostID":"{id}","displays":[{{"displayID":"d","input":70000}}]}}]}}"#,
            serde_json::to_string(&h).unwrap()
        );
        assert!(ConfigStore::decode(input.as_bytes()).is_err());
    }

    #[test]
    fn validation_rules() {
        let a = host("A", "a.local");
        let mut dup_label = AppConfig {
            hosts: vec![a.clone(), host("a", "b.local")],
            ..AppConfig::default()
        };
        assert!(dup_label.validate().is_err());
        dup_label.hosts[1].label = "B".into();
        assert!(dup_label.validate().is_ok());
        dup_label.hosts[1].hostname = " A.LOCAL ".into();
        assert!(
            dup_label.validate().is_err(),
            "duplicate hostname, case/space-insensitive"
        );
        dup_label.hosts[0].hostname = String::new();
        dup_label.hosts[1].hostname = String::new();
        assert!(dup_label.validate().is_ok(), "blank hostnames are ignored");

        let empty_label = AppConfig {
            hosts: vec![host("  ", "x")],
            ..AppConfig::default()
        };
        assert!(empty_label.validate().is_err());

        let mut orphan = with_profile(&a);
        orphan.hosts.clear();
        assert!(orphan.validate().is_err());

        let mut twice = with_profile(&a);
        let first = twice.profiles[0].clone();
        twice.profiles.push(first);
        assert!(twice.validate().is_err());

        let mut same_dev = with_profile(&a);
        same_dev.profiles[0].mouse = same_dev.profiles[0].keyboard.clone();
        assert!(same_dev.validate().is_err());

        let mut dup_display = with_profile(&a);
        let d = dup_display.profiles[0].displays[0].clone();
        dup_display.profiles[0].displays.push(d);
        assert!(dup_display.validate().is_err());
    }

    #[test]
    fn host_matching_prefers_label_then_hostname_then_uuid() {
        let a = host("Work", "mbp.local");
        let b = host("Home", "work");
        let c = AppConfig {
            hosts: vec![a.clone(), b.clone()],
            ..AppConfig::default()
        };
        assert_eq!(c.host_matching("WORK"), Some(&a));
        assert_eq!(c.host_matching("mbp.local"), Some(&a));
        assert_eq!(c.host_matching(&b.id.to_string().to_uppercase()), Some(&b));
        assert_eq!(c.host_matching("nope"), None);
    }

    #[test]
    fn remember_adds_updates_and_ignores_offline() {
        let mut c = AppConfig::default();
        let online = Peripheral::new(
            "x",
            "X",
            PeripheralKind::Keyboard,
            PeripheralTransport::Bolt,
        );
        assert!(c.remember(std::slice::from_ref(&online)));
        assert!(!c.remember(std::slice::from_ref(&online)), "unchanged");
        let renamed = Peripheral {
            name: "Y".into(),
            ..online.clone()
        };
        assert!(c.remember(&[renamed]));
        assert_eq!(c.known_devices[0].name, "Y");
        let offline = online.with_presence(DevicePresence::Offline);
        let other = Peripheral {
            id: "z".into(),
            ..offline
        };
        assert!(!c.remember(&[other]));
        assert_eq!(c.known_devices.len(), 1);
    }

    #[test]
    fn default_path_follows_xdg_with_home_fallback() {
        let home = PathBuf::from("/home/u");
        let path = |win, xdg: Option<&str>, appdata: Option<&str>| {
            ConfigStore::resolve_path(
                win,
                xdg.map(PathBuf::from),
                appdata.map(PathBuf::from),
                home.clone(),
            )
        };
        assert_eq!(
            path(false, None, None),
            PathBuf::from("/home/u/.config/hostswitch/config.json")
        );
        assert_eq!(
            path(false, Some("/x/cfg"), None),
            PathBuf::from("/x/cfg/hostswitch/config.json")
        );
        assert_eq!(
            path(false, Some("relative"), None),
            PathBuf::from("/home/u/.config/hostswitch/config.json")
        );
        assert_eq!(
            path(true, None, Some("C:/App")),
            PathBuf::from("C:/App/Hostswitch/config.json")
        );
    }

    /// `examples/config.example.json` is documented in the README; keep it loadable and valid.
    #[test]
    fn example_config_is_valid() {
        let json = include_str!("../../../examples/config.example.json");
        let c = ConfigStore::decode(json.as_bytes()).unwrap();
        assert_eq!(c.hosts.len(), 2);
        assert_eq!(c.profiles.len(), 2);
        assert_eq!(
            c.device_candidates("046d:b359/255"),
            ["046d:b359/255", "046d:c52b/1"]
        );
        assert!(c.host_matching("pc").is_some());
        // Every device a profile uses is a remembered one, so `show` prints names.
        for p in &c.profiles {
            for a in [&p.keyboard, &p.mouse].into_iter().flatten() {
                assert!(
                    c.known_devices.iter().any(|k| k.id == a.device_id),
                    "{}",
                    a.device_id
                );
            }
        }
    }

    #[test]
    fn device_aliases_round_trip_and_default_to_empty() {
        let mut c = AppConfig::default();
        assert!(
            !String::from_utf8(ConfigStore::encode(&c).unwrap())
                .unwrap()
                .contains("deviceAliases")
        );
        c.link_device("kb-ble", "kb-usb").unwrap();
        c.link_device("kb-ble", "kb-usb").unwrap(); // idempotent
        assert_eq!(c.device_candidates("kb-ble"), ["kb-ble", "kb-usb"]);
        assert_eq!(c.device_candidates("other"), ["other"]);
        let back = ConfigStore::decode(&ConfigStore::encode(&c).unwrap()).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn conflicting_aliases_are_rejected_and_leave_the_config_unchanged() {
        let mut c = AppConfig::default();
        c.link_device("a", "b").unwrap();
        let before = c.clone();
        for (primary, also) in [("a", "a"), ("c", "b"), ("b", "c"), ("c", "a"), ("c", " ")] {
            assert!(c.link_device(primary, also).is_err(), "{primary} -> {also}");
            assert_eq!(c, before);
        }
    }
}
