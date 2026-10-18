// SPDX-License-Identifier: Apache-2.0

//! Builds a starting config from detected hardware (`hostswitchctl init`). Pure: detection happens elsewhere.

use std::collections::HashMap;

use crate::config::AppConfig;
use crate::domain::{
    DeviceAssignment, Display, DisplayAssignment, DisplayInput, Host, KnownDevice, Peripheral,
    PeripheralKind, Profile, Slot,
};
use crate::error::{HostswitchError, Result};

#[derive(Debug, Clone)]
pub struct DetectedDisplay {
    pub display: Display,
    /// The input the display is on right now (None if it could not be read).
    pub input: Option<DisplayInput>,
}

#[derive(Debug, Clone, Default)]
pub struct Detected {
    pub displays: Vec<DetectedDisplay>,
    /// Devices that answered during discovery.
    pub devices: Vec<Peripheral>,
    /// Easy-Switch slot each device is on right now, where it could be read.
    pub slots: HashMap<String, Slot>,
}

/// Another computer on the desk; `input` is the display input that computer is plugged into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostSpec {
    pub label: String,
    pub input: Option<DisplayInput>,
}

#[derive(Debug, Clone)]
pub struct Bootstrap {
    pub config: AppConfig,
    /// Human-readable remarks: what was chosen, what is missing and how to fix it.
    pub notes: Vec<String>,
}

/// The first online device of `kind`, if any.
fn pick(devices: &[Peripheral], kind: PeripheralKind) -> Option<&Peripheral> {
    devices.iter().find(|d| d.is_online() && d.kind == kind)
}

/// `this_*` describes the computer running the tool: its displays at their current inputs and its
/// keyboard/mouse at their current slots. Each `others` entry gets the next free Easy-Switch slot
/// (the same slot for keyboard and mouse) and, if given, the display input it is plugged into.
pub fn bootstrap(
    this_label: &str,
    this_hostname: &str,
    others: &[HostSpec],
    detected: &Detected,
    existing_known: &[KnownDevice],
) -> Result<Bootstrap> {
    let mut notes = Vec::new();
    let keyboard = pick(&detected.devices, PeripheralKind::Keyboard);
    let mouse = pick(&detected.devices, PeripheralKind::Mouse);
    for (role, device) in [("keyboard", keyboard), ("mouse", mouse)] {
        match device {
            Some(d) => notes.push(format!("{role}: {} ({})", d.name, d.id)),
            None => notes.push(format!(
                "no online {role} detected: it is left out of the profiles"
            )),
        }
    }
    let unclassified = detected
        .devices
        .iter()
        .filter(|d| d.is_online() && d.kind == PeripheralKind::Unknown)
        .count();
    if unclassified > 0 {
        notes.push(format!(
            "{unclassified} online device(s) have an unknown kind (keyboard or mouse) and were not used"
        ));
    }

    let slot_of = |d: Option<&Peripheral>| d.and_then(|d| detected.slots.get(&d.id).copied());
    let this_slot = slot_of(keyboard)
        .or(slot_of(mouse))
        .or_else(|| Slot::new(Slot::MIN))
        .ok_or_else(|| HostswitchError::InvalidConfig("no valid slot".into()))?;
    if keyboard.is_some() && slot_of(keyboard).is_none() {
        notes.push(format!(
            "current slot of the keyboard could not be read: assuming {this_slot}"
        ));
    }

    let mut free_slots = Slot::all().filter(|s| *s != this_slot);
    let mut config = AppConfig {
        known_devices: existing_known.to_vec(),
        ..AppConfig::default()
    };
    config.remember(&detected.devices);

    let this_host = Host::new(this_label, this_hostname);
    let mut this_profile = Profile::new(this_host.id);
    for d in &detected.displays {
        match d.input {
            Some(input) => this_profile.displays.push(DisplayAssignment {
                display_id: d.display.id.clone(),
                input,
            }),
            None => notes.push(format!(
                "input of display '{}' could not be read: it is left out of '{this_label}'",
                d.display.name
            )),
        }
    }
    let assign = |d: Option<&Peripheral>, slot: Slot| {
        d.map(|d| DeviceAssignment {
            device_id: d.id.clone(),
            slot,
        })
    };
    this_profile.keyboard = assign(keyboard, this_slot);
    this_profile.mouse = assign(mouse, this_slot);
    config.hosts.push(this_host);
    config.profiles.push(this_profile);

    for spec in others {
        let slot = free_slots.next().ok_or_else(|| {
            HostswitchError::InvalidConfig(
                "Easy-Switch has 3 slots: this computer plus at most 2 other hosts".into(),
            )
        })?;
        let host = Host::new(&spec.label, "");
        let mut profile = Profile::new(host.id);
        match spec.input {
            Some(input) => profile.displays = detected
                .displays
                .iter()
                .map(|d| DisplayAssignment { display_id: d.display.id.clone(), input })
                .collect(),
            None => notes.push(format!(
                "'{}' has no display input: it only switches keyboard and mouse. Use --host \"{}=<input>\" (e.g. hdmi1)",
                spec.label, spec.label
            )),
        }
        profile.keyboard = assign(keyboard, slot);
        profile.mouse = assign(mouse, slot);
        config.hosts.push(host);
        config.profiles.push(profile);
    }
    if others.is_empty() {
        notes.push("no other hosts given: add them with --host \"<label>=<input>\"".into());
    }

    config.validate()?;
    Ok(Bootstrap { config, notes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::PeripheralTransport;

    fn display(id: &str, input: Option<u16>) -> DetectedDisplay {
        DetectedDisplay {
            display: Display {
                id: id.into(),
                name: format!("Monitor {id}"),
                backend_id: "t".into(),
            },
            input: input.map(DisplayInput),
        }
    }

    fn device(id: &str, kind: PeripheralKind) -> Peripheral {
        Peripheral::new(id, id, kind, PeripheralTransport::Ble)
    }

    fn detected() -> Detected {
        Detected {
            displays: vec![display("D1", Some(15))],
            devices: vec![
                device("kb", PeripheralKind::Keyboard),
                device("ms", PeripheralKind::Mouse),
            ],
            slots: HashMap::from([("kb".to_string(), Slot::new(2).unwrap())]),
        }
    }

    fn spec(label: &str, input: Option<u16>) -> HostSpec {
        HostSpec {
            label: label.into(),
            input: input.map(DisplayInput),
        }
    }

    #[test]
    fn this_computer_keeps_current_state_and_others_take_free_slots() {
        let b = bootstrap(
            "Mac",
            "mac.local",
            &[spec("PC", Some(17)), spec("Laptop", Some(18))],
            &detected(),
            &[],
        )
        .unwrap();
        let c = &b.config;
        assert_eq!(c.hosts.len(), 3);
        let p = |label: &str| c.profile(c.host_matching(label).unwrap().id).unwrap();
        // Mac is on slot 2 (read from the keyboard), the others get 1 and 3.
        assert_eq!(p("Mac").keyboard.as_ref().unwrap().slot.get(), 2);
        assert_eq!(p("Mac").mouse.as_ref().unwrap().slot.get(), 2);
        assert_eq!(p("Mac").displays[0].input, DisplayInput(15));
        assert_eq!(p("PC").keyboard.as_ref().unwrap().slot.get(), 1);
        assert_eq!(p("PC").displays[0].input, DisplayInput(17));
        assert_eq!(p("Laptop").keyboard.as_ref().unwrap().slot.get(), 3);
        assert_eq!(c.known_devices.len(), 2, "online devices are remembered");
        assert!(c.validate().is_ok());
    }

    #[test]
    fn missing_pieces_are_left_out_with_a_note() {
        let mut d = detected();
        d.displays.push(display("D2", None));
        d.devices.retain(|x| x.id != "ms");
        let b = bootstrap("Mac", "mac", &[spec("PC", None)], &d, &[]).unwrap();
        let mac = b.config.profile(b.config.hosts[0].id).unwrap();
        assert_eq!(mac.displays.len(), 1, "unreadable display input is omitted");
        assert!(mac.mouse.is_none());
        let pc = b.config.profile(b.config.hosts[1].id).unwrap();
        assert!(pc.displays.is_empty());
        let notes = b.notes.join("\n");
        assert!(notes.contains("no online mouse"), "{notes}");
        assert!(notes.contains("could not be read"), "{notes}");
        assert!(
            notes.contains("PC=<input>") || notes.contains("\"PC=<input>\""),
            "{notes}"
        );
    }

    #[test]
    fn unknown_kind_and_offline_devices_are_not_used() {
        let mut d = detected();
        d.devices = vec![
            device("u", PeripheralKind::Unknown),
            device("off", PeripheralKind::Keyboard)
                .with_presence(crate::domain::DevicePresence::Offline),
        ];
        let b = bootstrap("Mac", "mac", &[], &d, &[]).unwrap();
        let mac = b.config.profile(b.config.hosts[0].id).unwrap();
        assert!(mac.keyboard.is_none() && mac.mouse.is_none());
        assert!(b.notes.join("\n").contains("unknown kind"));
        assert!(
            b.config.known_devices.iter().all(|k| k.id != "off"),
            "offline devices are not newly remembered"
        );
    }

    #[test]
    fn unreadable_slot_falls_back_to_slot_one_with_a_note() {
        let mut d = detected();
        d.slots.clear();
        let b = bootstrap("Mac", "mac", &[spec("PC", Some(17))], &d, &[]).unwrap();
        let mac = b.config.profile(b.config.hosts[0].id).unwrap();
        assert_eq!(mac.keyboard.as_ref().unwrap().slot.get(), 1);
        assert_eq!(
            b.config
                .profile(b.config.hosts[1].id)
                .unwrap()
                .keyboard
                .as_ref()
                .unwrap()
                .slot
                .get(),
            2
        );
        assert!(b.notes.join("\n").contains("assuming slot 1"));
    }

    #[test]
    fn too_many_hosts_and_duplicate_labels_are_rejected() {
        let three = [spec("A", None), spec("B", None), spec("C", None)];
        assert!(bootstrap("Mac", "mac", &three, &detected(), &[]).is_err());
        assert!(
            bootstrap("Mac", "mac", &[spec("mac", None)], &detected(), &[]).is_err(),
            "label clash, case-insensitive"
        );
    }

    #[test]
    fn existing_known_devices_are_kept() {
        let old = KnownDevice {
            id: "old".into(),
            name: "Old".into(),
            kind: PeripheralKind::Keyboard,
            transport: PeripheralTransport::Unifying,
        };
        let b = bootstrap("Mac", "mac", &[], &detected(), std::slice::from_ref(&old)).unwrap();
        assert!(b.config.known_devices.contains(&old));
        assert_eq!(b.config.known_devices.len(), 3);
    }
}
