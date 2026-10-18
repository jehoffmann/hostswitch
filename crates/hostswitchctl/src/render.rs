// SPDX-License-Identifier: Apache-2.0

//! Everything hostswitchctl prints, as pure functions (no I/O) so the layout is unit-tested.
//!
//! Style: capitalised `Label:` columns (Display, Keyboard, Mouse), names with their id in parentheses,
//! values aligned, `OK` / `FAIL` / `SKIP` for results.

use std::collections::HashMap;
use std::fmt::Write;

use hostswitch_core::domain::{Peripheral, PeripheralKind, PeripheralTransport};
use hostswitch_core::{
    AppConfig, DisplayInput, ObservedState, Outcome, Slot, SwitchReport, SwitchStep, active_hosts,
};

pub const DISPLAY: &str = "Display:";
pub const KEYBOARD: &str = "Keyboard:";
pub const MOUSE: &str = "Mouse:";
/// Width of the longest role label, so values line up.
const ROLE_WIDTH: usize = 9;

pub fn slot_label(slot: Slot) -> String {
    format!("Slot {}", slot.get())
}

fn kind_label(kind: PeripheralKind) -> &'static str {
    match kind {
        PeripheralKind::Keyboard => "Keyboard",
        PeripheralKind::Mouse => "Mouse",
        PeripheralKind::Unknown => "Unknown",
    }
}

fn transport_label(transport: PeripheralTransport) -> &'static str {
    match transport {
        PeripheralTransport::Ble => "Bluetooth",
        PeripheralTransport::Unifying => "Unifying",
        PeripheralTransport::Bolt => "Bolt",
        PeripheralTransport::Unknown => "Unknown",
    }
}

fn with_id(name: Option<&str>, id: &str) -> String {
    match name {
        Some(name) => format!("{name} ({id})"),
        None => id.to_string(),
    }
}

/// A device as `Name (id)`; devices that are not remembered show the bare id.
fn device_label(config: &AppConfig, id: &str) -> String {
    let name = config
        .known_devices
        .iter()
        .find(|k| k.id == id)
        .map(|k| k.name.as_str());
    with_id(name, id)
}

fn width_of<'a>(items: impl Iterator<Item = &'a String>) -> usize {
    items.map(|s| s.chars().count()).max().unwrap_or(0)
}

// ---- list ----

pub struct DisplayRow {
    pub name: String,
    pub id: String,
    pub input: Option<DisplayInput>,
}

pub fn format_display_list(rows: &[DisplayRow]) -> String {
    if rows.is_empty() {
        return "  (none)\n".to_string();
    }
    let labels: Vec<String> = rows.iter().map(|r| with_id(Some(&r.name), &r.id)).collect();
    let w = width_of(labels.iter());
    let mut out = String::new();
    for (label, row) in labels.iter().zip(rows) {
        let input = row
            .input
            .map_or_else(|| "unknown".to_string(), |i| i.to_string());
        let _ = writeln!(out, "  {label:<w$}  Input: {input}");
    }
    out
}

pub fn format_device_list(devices: &[Peripheral]) -> String {
    if devices.is_empty() {
        return "  (none)\n".to_string();
    }
    let labels: Vec<String> = devices
        .iter()
        .map(|d| with_id(Some(&d.name), &d.id))
        .collect();
    let kinds: Vec<String> = devices
        .iter()
        .map(|d| format!("{}, {}", kind_label(d.kind), transport_label(d.transport)))
        .collect();
    let (lw, kw) = (width_of(labels.iter()), width_of(kinds.iter()));
    let mut out = String::new();
    for ((label, kind), d) in labels.iter().zip(&kinds).zip(devices) {
        let presence = if d.is_online() { "Online" } else { "Offline" };
        let _ = writeln!(out, "  {label:<lw$}  {kind:<kw$}  {presence}");
    }
    out
}

pub const PORTS_NOTE: &str = "  Note: m1ddc cannot read which ports this monitor really has, so this lists the standard DDC inputs.\n  Use the number in profiles (or --host LABEL=<number>); monitor-specific values also work.";

/// The selectable inputs for a display; `*` marks the current one. A current value without a standard
/// name is added to the list so it is never hidden.
pub fn format_ports(current: Option<DisplayInput>) -> String {
    let mut inputs = DisplayInput::KNOWN.to_vec();
    if let Some(c) = current.filter(|c| !inputs.contains(c)) {
        inputs.push(c);
    }
    let mut out = String::from("    Inputs (* = current):\n");
    for input in inputs {
        let mark = if Some(input) == current { '*' } else { ' ' };
        let _ = writeln!(out, "      {mark} {input}");
    }
    out
}

// ---- show ----

/// The configured hosts and what each profile sets. Reads the config only.
pub fn format_mapping(config: &AppConfig) -> String {
    if config.hosts.is_empty() {
        return "No hosts configured. Run `hostswitchctl init --host \"<label>=<input>\" --write`.\n"
            .to_string();
    }
    // One row per profile line, so the columns can be aligned across all hosts.
    struct Row {
        role: &'static str,
        target: String,
        value: String,
        extra: String,
    }
    let mut blocks = Vec::new();
    for host in &config.hosts {
        let mut rows = Vec::new();
        if let Some(profile) = config.profile(host.id) {
            for d in &profile.displays {
                rows.push(Row {
                    role: DISPLAY,
                    target: d.display_id.clone(),
                    value: d.input.to_string(),
                    extra: String::new(),
                });
            }
            for (role, assignment) in [(KEYBOARD, &profile.keyboard), (MOUSE, &profile.mouse)] {
                if let Some(a) = assignment {
                    let extra = match config.device_aliases.get(&a.device_id) {
                        Some(aliases) => format!("   (also via {})", aliases.join(", ")),
                        None => String::new(),
                    };
                    rows.push(Row {
                        role,
                        target: device_label(config, &a.device_id),
                        value: slot_label(a.slot),
                        extra,
                    });
                }
            }
        }
        blocks.push((host, rows));
    }
    let tw = blocks
        .iter()
        .flat_map(|(_, r)| r)
        .map(|r| r.target.chars().count())
        .max()
        .unwrap_or(0);

    let mut out = String::new();
    for (i, (host, rows)) in blocks.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if host.hostname.trim().is_empty() {
            let _ = writeln!(out, "{}", host.label);
        } else {
            let _ = writeln!(out, "{}  (hostname {})", host.label, host.hostname);
        }
        if config.profile(host.id).is_none() {
            out.push_str("  (no profile)\n");
        } else if rows.is_empty() {
            out.push_str("  (empty profile)\n");
        }
        for r in rows {
            let _ = writeln!(
                out,
                "  {:<ROLE_WIDTH$} {:<tw$} -> {}{}",
                r.role, r.target, r.value, r.extra
            );
        }
    }
    out
}

// ---- status ----

/// The configured hosts with their profiles, each value compared with the hardware's current state:
/// `OK` if it already matches, `now: <value>` if it differs, `now: unknown` if it could not be read.
/// `*` marks the active host. Names: displays from the backend, devices from the remembered devices.
pub fn format_status(
    config: &AppConfig,
    display_names: &HashMap<String, String>,
    observed: &ObservedState,
) -> String {
    let state = |matches: Option<bool>, now: Option<String>| match (matches, now) {
        (Some(true), _) => "OK".to_string(),
        (Some(false), Some(now)) => format!("now: {now}"),
        _ => "now: unknown".to_string(),
    };

    struct Row {
        role: &'static str,
        name: String,
        expected: String,
        state: String,
    }
    let active = active_hosts(config, observed);
    let mut blocks = Vec::new();
    for host in &config.hosts {
        let mut rows = Vec::new();
        if let Some(profile) = config.profile(host.id) {
            for d in &profile.displays {
                let now = observed.display_inputs.get(&d.display_id).copied();
                rows.push(Row {
                    role: DISPLAY,
                    name: with_id(
                        display_names.get(&d.display_id).map(String::as_str),
                        &d.display_id,
                    ),
                    expected: d.input.to_string(),
                    state: state(now.map(|n| n == d.input), now.map(|n| n.to_string())),
                });
            }
            for (role, assignment) in [(KEYBOARD, &profile.keyboard), (MOUSE, &profile.mouse)] {
                if let Some(a) = assignment {
                    let now = observed.device_slots.get(&a.device_id).copied();
                    rows.push(Row {
                        role,
                        name: device_label(config, &a.device_id),
                        expected: slot_label(a.slot),
                        state: state(now.map(|n| n == a.slot), now.map(slot_label)),
                    });
                }
            }
        }
        blocks.push((host, rows));
    }

    let nw = blocks
        .iter()
        .flat_map(|(_, r)| r)
        .map(|r| r.name.chars().count())
        .max()
        .unwrap_or(0);
    let ew = blocks
        .iter()
        .flat_map(|(_, r)| r)
        .map(|r| r.expected.chars().count())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for (host, rows) in &blocks {
        let marker = if active.contains(host) { '*' } else { ' ' };
        let _ = writeln!(out, "{marker} {}", host.label);
        if config.profile(host.id).is_none() {
            out.push_str("    (no profile)\n");
        } else if rows.is_empty() {
            out.push_str("    (empty profile)\n");
        }
        for r in rows {
            let _ = writeln!(
                out,
                "    {:<ROLE_WIDTH$} {:<nw$}  {:<ew$}  {}",
                r.role, r.name, r.expected, r.state
            );
        }
    }
    if config.hosts.is_empty() {
        out.push_str("(no hosts configured)\n");
    }
    out
}

// ---- switch ----

/// A step as `(role, "target -> value")`.
pub fn describe_step(step: &SwitchStep, config: &AppConfig) -> (&'static str, String) {
    match step {
        SwitchStep::Display { display_id, input } => (DISPLAY, format!("{display_id} -> {input}")),
        SwitchStep::Keyboard { device_id, slot } => (
            KEYBOARD,
            format!(
                "{} -> {}",
                device_label(config, device_id),
                slot_label(*slot)
            ),
        ),
        SwitchStep::Mouse { device_id, slot } => (
            MOUSE,
            format!(
                "{} -> {}",
                device_label(config, device_id),
                slot_label(*slot)
            ),
        ),
    }
}

pub fn format_plan(host_label: &str, steps: &[SwitchStep], config: &AppConfig) -> String {
    let mut out = format!("Plan for '{host_label}':\n");
    for (i, step) in steps.iter().enumerate() {
        let (role, text) = describe_step(step, config);
        let _ = writeln!(out, "  {}. {role:<ROLE_WIDTH$} {text}", i + 1);
    }
    out
}

pub fn format_report(report: &SwitchReport, config: &AppConfig) -> String {
    let mut out = String::new();
    for r in &report.results {
        let (role, text) = describe_step(&r.step, config);
        let line = match &r.outcome {
            Outcome::Success => format!("  OK    {role:<ROLE_WIDTH$} {text}"),
            Outcome::Failure(message) => {
                format!("  FAIL  {role:<ROLE_WIDTH$} {text}: {message}")
            }
            Outcome::Skipped(reason) => format!("  SKIP  {role:<ROLE_WIDTH$} {text}: {reason}"),
        };
        out.push_str(&line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hostswitch_core::{
        DeviceAssignment, DisplayAssignment, Host, KnownDevice, Profile, StepResult,
    };

    fn slot(n: u8) -> Slot {
        Slot::new(n).unwrap()
    }

    fn sample_config() -> AppConfig {
        let mac = Host::new("Mac", "mac.local");
        let pc = Host::new("PC", "");
        let bare = Host::new("Bare", "bare");
        let mut p = Profile::new(mac.id);
        p.displays.push(DisplayAssignment {
            display_id: "D1".into(),
            input: DisplayInput(15),
        });
        p.keyboard = Some(DeviceAssignment {
            device_id: "kb".into(),
            slot: slot(1),
        });
        p.mouse = Some(DeviceAssignment {
            device_id: "gone".into(),
            slot: slot(1),
        });
        let mut q = Profile::new(pc.id);
        q.displays.push(DisplayAssignment {
            display_id: "D1".into(),
            input: DisplayInput(17),
        });
        AppConfig {
            hosts: vec![mac, pc, bare.clone()],
            profiles: vec![p, q, Profile::new(bare.id)],
            known_devices: vec![KnownDevice {
                id: "kb".into(),
                name: "ERGO K860".into(),
                kind: PeripheralKind::Keyboard,
                transport: PeripheralTransport::Ble,
            }],
            ..AppConfig::default()
        }
    }

    #[test]
    fn show_uses_capitalised_labels_and_aligns_values() {
        let expected = "\
Mac  (hostname mac.local)
  Display:  D1             -> DP1 (15)
  Keyboard: ERGO K860 (kb) -> Slot 1
  Mouse:    gone           -> Slot 1

PC
  Display:  D1             -> HDMI1 (17)

Bare  (hostname bare)
  (empty profile)
";
        assert_eq!(format_mapping(&sample_config()), expected);
    }

    #[test]
    fn show_lists_aliases_and_handles_missing_profile_and_no_hosts() {
        let mut c = sample_config();
        c.link_device("kb", "kb-usb").unwrap();
        assert!(
            format_mapping(&c).contains("-> Slot 1   (also via kb-usb)"),
            "{}",
            format_mapping(&c)
        );
        c.profiles.truncate(1);
        assert!(format_mapping(&c).contains("PC\n  (no profile)\n"));
        assert!(format_mapping(&AppConfig::default()).contains("No hosts configured"));
    }

    #[test]
    fn status_groups_by_host_and_compares_with_the_hardware() {
        let config = sample_config();
        let mut observed = ObservedState::default();
        observed
            .display_inputs
            .insert("D1".into(), DisplayInput(15));
        observed.device_slots.insert("kb".into(), slot(1));
        // "gone" (the Mac's mouse) was not readable; Bare has an empty profile; PC has D1 -> 17.
        let names = HashMap::from([("D1".to_string(), "MSI".to_string())]);
        let expected = "\
* Mac
    Display:  MSI (D1)        DP1 (15)    OK
    Keyboard: ERGO K860 (kb)  Slot 1      OK
    Mouse:    gone            Slot 1      now: unknown
  PC
    Display:  MSI (D1)        HDMI1 (17)  now: DP1 (15)
  Bare
    (empty profile)
";
        assert_eq!(format_status(&config, &names, &observed), expected);
    }

    #[test]
    fn status_shows_the_current_slot_when_it_differs() {
        let config = sample_config();
        let mut observed = ObservedState::default();
        observed.device_slots.insert("kb".into(), slot(3));
        let text = format_status(&config, &HashMap::new(), &observed);
        assert!(text.contains("Slot 1      now: Slot 3"), "{text}");
    }

    #[test]
    fn status_handles_missing_profile_and_no_hosts() {
        let mut c = sample_config();
        c.profiles.truncate(1);
        let text = format_status(&c, &HashMap::new(), &ObservedState::default());
        assert!(text.contains("  PC\n    (no profile)\n"), "{text}");
        assert_eq!(
            format_status(
                &AppConfig::default(),
                &HashMap::new(),
                &ObservedState::default()
            ),
            "(no hosts configured)\n"
        );
    }

    #[test]
    fn display_list_aligns_and_handles_unknown_input_and_empty() {
        let rows = [
            DisplayRow {
                name: "MSI".into(),
                id: "U1".into(),
                input: Some(DisplayInput(15)),
            },
            DisplayRow {
                name: "Dell U2723QE".into(),
                id: "U2".into(),
                input: None,
            },
        ];
        assert_eq!(
            format_display_list(&rows),
            "  MSI (U1)           Input: DP1 (15)\n  Dell U2723QE (U2)  Input: unknown\n"
        );
        assert_eq!(format_display_list(&[]), "  (none)\n");
    }

    #[test]
    fn device_list_has_readable_kinds_transports_and_no_vendor_name() {
        let online = Peripheral::new(
            "046d:b020/255",
            "MX Vertical",
            PeripheralKind::Mouse,
            PeripheralTransport::Ble,
        );
        let offline = Peripheral::new(
            "046d:c52b/1",
            "Unifying device 1",
            PeripheralKind::Unknown,
            PeripheralTransport::Unifying,
        )
        .with_presence(hostswitch_core::domain::DevicePresence::Offline);
        let text = format_device_list(&[online, offline]);
        assert_eq!(
            text,
            "  MX Vertical (046d:b020/255)      Mouse, Bluetooth   Online\n  Unifying device 1 (046d:c52b/1)  Unknown, Unifying  Offline\n"
        );
        assert!(!text.to_lowercase().contains("logitech"));
        assert_eq!(format_device_list(&[]), "  (none)\n");
    }

    #[test]
    fn ports_list_marks_the_current_input() {
        let text = format_ports(Some(DisplayInput(15)));
        assert!(text.contains("      * DP1 (15)\n"), "{text}");
        assert!(text.contains("        HDMI1 (17)\n"), "{text}");
        assert_eq!(
            text.matches('*').count(),
            2,
            "one marker plus the legend: {text}"
        );
        assert_eq!(text.lines().count(), 1 + DisplayInput::KNOWN.len());
    }

    #[test]
    fn ports_list_keeps_an_unnamed_current_input_and_handles_unknown() {
        let text = format_ports(Some(DisplayInput(208)));
        assert!(text.contains("      * input 208\n"), "{text}");
        assert_eq!(text.lines().count(), 2 + DisplayInput::KNOWN.len());
        let unknown = format_ports(None);
        assert_eq!(unknown.matches('*').count(), 1, "legend only: {unknown}");
    }

    fn steps() -> Vec<SwitchStep> {
        vec![
            SwitchStep::Display {
                display_id: "D1".into(),
                input: DisplayInput(15),
            },
            SwitchStep::Keyboard {
                device_id: "kb".into(),
                slot: slot(2),
            },
            SwitchStep::Mouse {
                device_id: "gone".into(),
                slot: slot(2),
            },
        ]
    }

    #[test]
    fn plan_uses_names_and_capitalised_roles() {
        assert_eq!(
            format_plan("Work", &steps(), &sample_config()),
            "Plan for 'Work':\n  1. Display:  D1 -> DP1 (15)\n  2. Keyboard: ERGO K860 (kb) -> Slot 2\n  3. Mouse:    gone -> Slot 2\n"
        );
    }

    #[test]
    fn report_lines_show_ok_fail_skip() {
        let outcomes = [
            Outcome::Success,
            Outcome::Failure("device offline".into()),
            Outcome::Skipped("because".into()),
        ];
        let report = SwitchReport {
            results: steps()
                .into_iter()
                .zip(outcomes)
                .map(|(step, outcome)| StepResult {
                    step,
                    outcome,
                    attempts: 1,
                })
                .collect(),
        };
        assert_eq!(
            format_report(&report, &sample_config()),
            "  OK    Display:  D1 -> DP1 (15)\n  FAIL  Keyboard: ERGO K860 (kb) -> Slot 2: device offline\n  SKIP  Mouse:    gone -> Slot 2: because\n"
        );
    }
}
