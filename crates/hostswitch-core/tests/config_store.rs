// SPDX-License-Identifier: Apache-2.0

//! Integration tests: `ConfigStore` reads and writes real files in a temporary directory.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use hostswitch_core::{
    AppConfig, ConfigStore, DeviceAssignment, DisplayAssignment, DisplayInput, Host, Profile, Slot,
};
use uuid::Uuid;

fn config_with_profile() -> AppConfig {
    let host = Host::new("Mac", "mac");
    let mut profile = Profile::new(host.id);
    profile.displays.push(DisplayAssignment {
        display_id: "D1".into(),
        input: DisplayInput(15),
    });
    profile.keyboard = Some(DeviceAssignment {
        device_id: "046d:c52b/1".into(),
        slot: Slot::new(1).unwrap(),
    });
    AppConfig {
        hosts: vec![host],
        profiles: vec![profile],
        ..AppConfig::default()
    }
}

#[test]
fn save_is_atomic_and_keeps_old_file_on_rejection() {
    let dir = std::env::temp_dir().join(format!("hostswitch-test-{}", Uuid::new_v4()));
    let store = ConfigStore::new(dir.join("nested/config.json"));
    assert_eq!(
        store.load().unwrap(),
        AppConfig::default(),
        "missing file = default"
    );
    let good = config_with_profile();
    store.save(&good).unwrap();
    assert_eq!(store.load().unwrap(), good);
    let bad = AppConfig {
        version: 9,
        ..good.clone()
    };
    assert!(store.save(&bad).is_err());
    assert_eq!(store.load().unwrap(), good);
    std::fs::remove_dir_all(dir).unwrap();
}
