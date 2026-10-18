// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use crate::domain::{DisplayInput, Profile, Slot};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwitchStep {
    Display {
        display_id: String,
        input: DisplayInput,
    },
    Keyboard {
        device_id: String,
        slot: Slot,
    },
    Mouse {
        device_id: String,
        slot: Slot,
    },
}

impl SwitchStep {
    pub fn is_peripheral(&self) -> bool {
        !matches!(self, Self::Display { .. })
    }
}

impl fmt::Display for SwitchStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Display { display_id, input } => write!(f, "display {display_id} -> {input}"),
            Self::Keyboard { device_id, slot } => write!(f, "keyboard {device_id} -> {slot}"),
            Self::Mouse { device_id, slot } => write!(f, "mouse {device_id} -> {slot}"),
        }
    }
}

/// Pure ordering logic. Displays first, then keyboard, then mouse: losing the keyboard
/// first would remove the user's ability to react if a display step fails.
pub fn plan(profile: &Profile) -> Vec<SwitchStep> {
    let mut steps: Vec<SwitchStep> = profile
        .displays
        .iter()
        .map(|d| SwitchStep::Display {
            display_id: d.display_id.clone(),
            input: d.input,
        })
        .collect();
    if let Some(kb) = &profile.keyboard {
        steps.push(SwitchStep::Keyboard {
            device_id: kb.device_id.clone(),
            slot: kb.slot,
        });
    }
    if let Some(m) = &profile.mouse {
        steps.push(SwitchStep::Mouse {
            device_id: m.device_id.clone(),
            slot: m.slot,
        });
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DeviceAssignment, DisplayAssignment};
    use uuid::Uuid;

    fn slot(n: u8) -> Slot {
        Slot::new(n).unwrap()
    }

    #[test]
    fn orders_displays_then_keyboard_then_mouse() {
        let mut p = Profile::new(Uuid::new_v4());
        p.mouse = Some(DeviceAssignment {
            device_id: "m".into(),
            slot: slot(2),
        });
        p.keyboard = Some(DeviceAssignment {
            device_id: "k".into(),
            slot: slot(1),
        });
        p.displays = vec![
            DisplayAssignment {
                display_id: "d1".into(),
                input: DisplayInput(15),
            },
            DisplayAssignment {
                display_id: "d2".into(),
                input: DisplayInput(17),
            },
        ];
        let steps = plan(&p);
        assert_eq!(steps.len(), 4);
        assert!(matches!(steps[0], SwitchStep::Display { .. }));
        assert!(matches!(steps[1], SwitchStep::Display { .. }));
        assert!(matches!(steps[2], SwitchStep::Keyboard { .. }));
        assert!(matches!(steps[3], SwitchStep::Mouse { .. }));
    }

    #[test]
    fn optional_parts_and_empty_profile() {
        assert!(plan(&Profile::new(Uuid::new_v4())).is_empty());
        let mut p = Profile::new(Uuid::new_v4());
        p.mouse = Some(DeviceAssignment {
            device_id: "m".into(),
            slot: slot(3),
        });
        assert_eq!(
            plan(&p),
            vec![SwitchStep::Mouse {
                device_id: "m".into(),
                slot: slot(3)
            }]
        );
    }

    #[test]
    fn step_descriptions_are_readable() {
        let s = SwitchStep::Display {
            display_id: "D".into(),
            input: DisplayInput(15),
        };
        assert_eq!(s.to_string(), "display D -> DP1 (15)");
        let k = SwitchStep::Keyboard {
            device_id: "k".into(),
            slot: slot(2),
        };
        assert_eq!(k.to_string(), "keyboard k -> slot 2");
    }

    #[test]
    fn five_displays_plan_five_display_steps() {
        let mut p = Profile::new(Uuid::new_v4());
        p.displays = (1..=5)
            .map(|n| DisplayAssignment {
                display_id: format!("d{n}"),
                input: DisplayInput(10 + n),
            })
            .collect();
        let steps = plan(&p);
        assert_eq!(steps.len(), 5);
        assert!(
            steps
                .iter()
                .all(|s| matches!(s, SwitchStep::Display { .. }))
        );
        assert_eq!(
            steps[4],
            SwitchStep::Display {
                display_id: "d5".into(),
                input: DisplayInput(15)
            }
        );
    }
}
