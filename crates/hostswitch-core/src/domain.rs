// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DDC/CI VCP 0x60 value (e.g. 15 = DP1, 17 = HDMI1, 27 = USB-C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DisplayInput(pub u16);

impl DisplayInput {
    pub const DISPLAY_PORT_1: Self = Self(15);
    pub const DISPLAY_PORT_2: Self = Self(16);
    pub const HDMI_1: Self = Self(17);
    pub const HDMI_2: Self = Self(18);
    pub const USB_C: Self = Self(27);
    /// Inputs offered first in pick-lists.
    pub const COMMON: [Self; 5] = [
        Self::DISPLAY_PORT_1,
        Self::DISPLAY_PORT_2,
        Self::HDMI_1,
        Self::HDMI_2,
        Self::USB_C,
    ];

    /// Every input value with a standard name (MCCS VCP 0x60; 15-18 and 27 as listed by m1ddc), in
    /// ascending order. A monitor supports only a subset, and some use vendor-specific numbers.
    pub const KNOWN: [Self; 9] = [
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self::DISPLAY_PORT_1,
        Self::DISPLAY_PORT_2,
        Self::HDMI_1,
        Self::HDMI_2,
        Self::USB_C,
    ];

    /// Parses a number (`17`) or a common name (`dp1`, `dp2`, `hdmi1`, `hdmi2`, `usbc`), case-insensitive.
    pub fn parse(text: &str) -> Option<Self> {
        let t = text.trim().to_lowercase();
        match t.as_str() {
            "vga1" => Some(Self(1)),
            "vga2" => Some(Self(2)),
            "dvi1" => Some(Self(3)),
            "dvi2" => Some(Self(4)),
            "dp1" => Some(Self::DISPLAY_PORT_1),
            "dp2" => Some(Self::DISPLAY_PORT_2),
            "hdmi1" => Some(Self::HDMI_1),
            "hdmi2" => Some(Self::HDMI_2),
            "usbc" | "usb-c" => Some(Self::USB_C),
            _ => t.parse::<u16>().ok().map(Self),
        }
    }
}

impl fmt::Display for DisplayInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            1 => write!(f, "VGA1 (1)"),
            2 => write!(f, "VGA2 (2)"),
            3 => write!(f, "DVI1 (3)"),
            4 => write!(f, "DVI2 (4)"),
            15 => write!(f, "DP1 (15)"),
            16 => write!(f, "DP2 (16)"),
            17 => write!(f, "HDMI1 (17)"),
            18 => write!(f, "HDMI2 (18)"),
            27 => write!(f, "USB-C (27)"),
            n => write!(f, "input {n}"),
        }
    }
}

/// Logitech Easy-Switch host index, 1...3 (user-facing, 1-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "u8")]
pub struct Slot(u8);

impl Slot {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 3;

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    pub fn get(self) -> u8 {
        self.0
    }

    /// Every valid slot, in order.
    pub fn all() -> impl Iterator<Item = Self> {
        (Self::MIN..=Self::MAX).map(Self)
    }
}

impl TryFrom<i64> for Slot {
    type Error = String;
    fn try_from(value: i64) -> std::result::Result<Self, String> {
        u8::try_from(value)
            .ok()
            .and_then(Self::new)
            .ok_or_else(|| format!("slot {value} must be in {}...{}", Self::MIN, Self::MAX))
    }
}

impl From<Slot> for u8 {
    fn from(slot: Slot) -> u8 {
        slot.0
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "slot {}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Display {
    pub id: String,
    pub name: String,
    #[serde(rename = "backendID")]
    pub backend_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeripheralKind {
    Keyboard,
    Mouse,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeripheralTransport {
    Bolt,
    Unifying,
    Ble,
    Unknown,
}

impl PeripheralKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Unknown => "unknown",
        }
    }
}

impl PeripheralTransport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bolt => "bolt",
            Self::Unifying => "unifying",
            Self::Ble => "ble",
            Self::Unknown => "unknown",
        }
    }
}

/// Whether a keyboard/mouse device answered a genuine ping echo during the last discovery (F-13).
/// Offline covers "asleep", "connected to another host" and (indistinguishably, R-6) "slot empty".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DevicePresence {
    Online,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Peripheral {
    pub id: String,
    pub name: String,
    pub kind: PeripheralKind,
    pub transport: PeripheralTransport,
    pub presence: DevicePresence,
}

impl Peripheral {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: PeripheralKind,
        transport: PeripheralTransport,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            transport,
            presence: DevicePresence::Online,
        }
    }

    pub fn is_online(&self) -> bool {
        self.presence == DevicePresence::Online
    }

    pub fn with_presence(mut self, presence: DevicePresence) -> Self {
        self.presence = presence;
        self
    }
}

/// A device remembered from an earlier discovery so it can be shown (as offline) and referenced by
/// profiles while it sleeps or is connected elsewhere (F-13, R-6). The receiver and slot index are
/// part of `id` ("vid:pid/index" for the hidapitester backend).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KnownDevice {
    pub id: String,
    pub name: String,
    pub kind: PeripheralKind,
    pub transport: PeripheralTransport,
}

impl From<&Peripheral> for KnownDevice {
    fn from(d: &Peripheral) -> Self {
        Self {
            id: d.id.clone(),
            name: d.name.clone(),
            kind: d.kind,
            transport: d.transport,
        }
    }
}

impl KnownDevice {
    pub fn as_offline_device(&self) -> Peripheral {
        Peripheral {
            id: self.id.clone(),
            name: self.name.clone(),
            kind: self.kind,
            transport: self.transport,
            presence: DevicePresence::Offline,
        }
    }
}

pub struct DeviceCatalog;

impl DeviceCatalog {
    /// Live devices (online) plus remembered ones that did not answer (offline), live first.
    pub fn merge(live: &[Peripheral], known: &[KnownDevice]) -> Vec<Peripheral> {
        let mut merged: Vec<Peripheral> = live
            .iter()
            .cloned()
            .map(|d| d.with_presence(DevicePresence::Online))
            .collect();
        merged.extend(
            known
                .iter()
                .filter(|k| !live.iter().any(|d| d.id == k.id))
                .map(KnownDevice::as_offline_device),
        );
        merged
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Host {
    pub id: Uuid,
    pub label: String,
    pub hostname: String,
}

impl Host {
    pub fn new(label: impl Into<String>, hostname: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            hostname: hostname.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayAssignment {
    #[serde(rename = "displayID")]
    pub display_id: String,
    pub input: DisplayInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceAssignment {
    #[serde(rename = "deviceID")]
    pub device_id: String,
    pub slot: Slot,
}

/// What the desk should look like when `host_id` is the active computer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "hostID")]
    pub host_id: Uuid,
    #[serde(default)]
    pub displays: Vec<DisplayAssignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<DeviceAssignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mouse: Option<DeviceAssignment>,
}

impl Profile {
    pub fn new(host_id: Uuid) -> Self {
        Self {
            host_id,
            displays: Vec::new(),
            keyboard: None,
            mouse: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_rejects_out_of_range() {
        assert!(Slot::new(0).is_none());
        assert!(Slot::new(4).is_none());
        assert_eq!(Slot::new(3).map(Slot::get), Some(3));
        assert!(serde_json::from_str::<Slot>("4").is_err());
        assert!(serde_json::from_str::<Slot>("-1").is_err());
        assert!(serde_json::from_str::<Slot>("1000").is_err());
        assert_eq!(
            serde_json::from_str::<Slot>("2").ok().map(Slot::get),
            Some(2)
        );
    }

    #[test]
    fn display_input_parses_numbers_and_names() {
        assert_eq!(DisplayInput::parse("17"), Some(DisplayInput(17)));
        assert_eq!(DisplayInput::parse(" HDMI1 "), Some(DisplayInput(17)));
        assert_eq!(DisplayInput::parse("usb-c"), Some(DisplayInput(27)));
        assert_eq!(DisplayInput::parse("dp2"), Some(DisplayInput(16)));
        for bad in ["", "dp3", "-1", "70000", "x"] {
            assert_eq!(DisplayInput::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn known_inputs_are_sorted_named_and_parseable() {
        assert!(DisplayInput::KNOWN.windows(2).all(|w| w[0] < w[1]));
        for input in DisplayInput::KNOWN {
            assert!(
                !input.to_string().starts_with("input "),
                "{input:?} has a name"
            );
        }
        assert_eq!(DisplayInput::parse("dvi2"), Some(DisplayInput(4)));
        assert_eq!(DisplayInput::parse("VGA1"), Some(DisplayInput(1)));
    }

    #[test]
    fn display_input_range_and_names() {
        assert!(serde_json::from_str::<DisplayInput>("70000").is_err());
        assert!(serde_json::from_str::<DisplayInput>("-1").is_err());
        assert_eq!(DisplayInput(15).to_string(), "DP1 (15)");
        assert_eq!(DisplayInput(27).to_string(), "USB-C (27)");
        assert_eq!(DisplayInput(99).to_string(), "input 99");
    }

    #[test]
    fn merge_puts_live_first_and_marks_silent_known_offline() {
        let live = [Peripheral::new(
            "a",
            "A",
            PeripheralKind::Keyboard,
            PeripheralTransport::Bolt,
        )];
        let known = [
            KnownDevice::from(&live[0]),
            KnownDevice {
                id: "b".into(),
                name: "B".into(),
                kind: PeripheralKind::Mouse,
                transport: PeripheralTransport::Unifying,
            },
        ];
        let merged = DeviceCatalog::merge(&live, &known);
        assert_eq!(merged.len(), 2);
        assert!(merged[0].is_online());
        assert_eq!(merged[1].id, "b");
        assert!(!merged[1].is_online());
    }
}
