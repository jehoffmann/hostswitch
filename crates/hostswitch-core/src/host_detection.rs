// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::config::AppConfig;
use crate::domain::{DisplayInput, Host, Slot};

/// Observed hardware state, as read from backends.
#[derive(Debug, Clone, Default)]
pub struct ObservedState {
    pub display_inputs: HashMap<String, DisplayInput>,
    pub device_slots: HashMap<String, Slot>,
}

/// A host is "active" when every observed part of its profile matches. Unobserved parts are ignored,
/// but at least one part must have been observed and matched.
pub fn active_hosts<'a>(config: &'a AppConfig, observed: &ObservedState) -> Vec<&'a Host> {
    config
        .hosts
        .iter()
        .filter(|host| {
            let Some(profile) = config.profile(host.id) else {
                return false;
            };
            let mut matched = 0;
            for d in &profile.displays {
                if let Some(current) = observed.display_inputs.get(&d.display_id) {
                    if *current != d.input {
                        return false;
                    }
                    matched += 1;
                }
            }
            for a in [&profile.keyboard, &profile.mouse].into_iter().flatten() {
                if let Some(current) = observed.device_slots.get(&a.device_id) {
                    if *current != a.slot {
                        return false;
                    }
                    matched += 1;
                }
            }
            matched > 0
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DeviceAssignment, DisplayAssignment, Profile};

    fn config() -> (AppConfig, Host, Host) {
        let a = Host::new("A", "a");
        let b = Host::new("B", "b");
        let mut pa = Profile::new(a.id);
        pa.displays.push(DisplayAssignment {
            display_id: "d".into(),
            input: DisplayInput(15),
        });
        pa.keyboard = Some(DeviceAssignment {
            device_id: "k".into(),
            slot: Slot::new(1).unwrap(),
        });
        let mut pb = Profile::new(b.id);
        pb.displays.push(DisplayAssignment {
            display_id: "d".into(),
            input: DisplayInput(17),
        });
        pb.keyboard = Some(DeviceAssignment {
            device_id: "k".into(),
            slot: Slot::new(2).unwrap(),
        });
        let c = AppConfig {
            hosts: vec![a.clone(), b.clone()],
            profiles: vec![pa, pb],
            ..AppConfig::default()
        };
        (c, a, b)
    }

    #[test]
    fn matches_the_host_whose_parts_all_agree() {
        let (c, a, _) = config();
        let mut o = ObservedState::default();
        o.display_inputs.insert("d".into(), DisplayInput(15));
        o.device_slots.insert("k".into(), Slot::new(1).unwrap());
        assert_eq!(active_hosts(&c, &o), vec![&a]);
    }

    #[test]
    fn mixed_state_matches_nobody_and_nothing_observed_matches_nobody() {
        let (c, _, _) = config();
        let mut o = ObservedState::default();
        assert!(active_hosts(&c, &o).is_empty());
        o.display_inputs.insert("d".into(), DisplayInput(15));
        o.device_slots.insert("k".into(), Slot::new(2).unwrap());
        assert!(active_hosts(&c, &o).is_empty());
    }

    #[test]
    fn partial_observation_can_match_several_hosts_r_t4() {
        let (c, a, b) = config();
        let mut o = ObservedState::default();
        o.display_inputs.insert("other".into(), DisplayInput(1));
        assert!(active_hosts(&c, &o).is_empty());
        o.device_slots.insert("k".into(), Slot::new(1).unwrap());
        assert_eq!(active_hosts(&c, &o), vec![&a]);
        let _ = b;
    }
}
