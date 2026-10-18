// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use hostswitch_core::DisplayInput;
use hostswitch_core::bootstrap::HostSpec;

/// Switch display inputs (DDC) and keyboard/mouse Easy-Switch slots together.
#[derive(Debug, Parser)]
#[command(name = "hostswitchctl", version)]
pub struct Cli {
    /// Config file (default: the platform's per-user config directory).
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show detected displays and keyboard/mouse devices (read-only).
    List {
        #[arg(value_enum)]
        what: Option<ListTarget>,
        /// With displays: also list the selectable input ports (the current one is marked).
        #[arg(long)]
        ports: bool,
    },
    /// Print the configured hosts and their mapping (reads the config file only; no hardware access).
    Show,
    /// Declare another way to reach a device, used when its main id is offline (e.g. Bluetooth id first,
    /// then the Unifying receiver id). Saves the config.
    Link {
        /// The device id used in profiles, e.g. 046d:b359/255
        device: String,
        /// Another id of the same physical device, e.g. 046d:c52b/1
        also: String,
    },
    /// Show current inputs/slots and which configured host is active.
    Status,
    /// Build a starting config from the detected displays and online devices (prints it; --write saves).
    Init(InitArgs),
    /// Switch displays and keyboard/mouse devices to a host's profile (hardware!). With --dry-run only print the plan.
    Switch(SwitchArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListTarget {
    Displays,
    Devices,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Another computer on the desk: LABEL or LABEL=INPUT, where INPUT is the display input that
    /// computer is plugged into (a number or dp1, dp2, hdmi1, hdmi2, usbc). Repeat for each (max 2).
    #[arg(long = "host", value_name = "LABEL[=INPUT]", value_parser = parse_host_spec)]
    pub hosts: Vec<HostSpec>,
    /// Label for this computer (default: the short hostname).
    #[arg(long)]
    pub label: Option<String>,
    /// Save the config instead of only printing it.
    #[arg(long)]
    pub write: bool,
    /// With --write: replace existing hosts and profiles (remembered devices are kept).
    #[arg(long, requires = "write")]
    pub force: bool,
}

pub fn parse_host_spec(text: &str) -> Result<HostSpec, String> {
    let (label, input) = match text.split_once('=') {
        Some((label, input)) => {
            let input = DisplayInput::parse(input).ok_or_else(|| {
                format!(
                    "unknown display input '{input}' (use a number or dp1, dp2, hdmi1, hdmi2, usbc)"
                )
            })?;
            (label, Some(input))
        }
        None => (text, None),
    };
    let label = label.trim();
    if label.is_empty() {
        return Err("host label must not be empty".into());
    }
    Ok(HostSpec {
        label: label.to_string(),
        input,
    })
}

#[derive(Debug, Args)]
pub struct SwitchArgs {
    /// Host label, hostname or UUID.
    pub host: String,
    /// Only print the plan; change nothing.
    #[arg(long, alias = "dry")]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("hostswitchctl").chain(args.iter().copied()))
    }

    #[test]
    fn switch_really_switches_by_default() {
        let Command::Switch(s) = parse(&["switch", "Mac"]).unwrap().command else {
            panic!("not switch")
        };
        assert_eq!((s.host.as_str(), s.dry_run), ("Mac", false));
    }

    #[test]
    fn dry_run_has_a_short_alias_and_execute_is_gone() {
        for flag in ["--dry-run", "--dry"] {
            let Command::Switch(s) = parse(&["switch", "Mac", flag]).unwrap().command else {
                panic!()
            };
            assert!(s.dry_run, "{flag}");
        }
        assert!(parse(&["switch", "Mac", "--execute"]).is_err());
    }

    #[test]
    fn config_flag_works_before_and_after_the_subcommand() {
        assert_eq!(
            parse(&["--config", "/x.json", "status"]).unwrap().config,
            Some("/x.json".into())
        );
        assert_eq!(
            parse(&["status", "--config", "/y.json"]).unwrap().config,
            Some("/y.json".into())
        );
        assert!(parse(&["status", "--config"]).is_err(), "missing value");
    }

    #[test]
    fn rejects_unknown_input() {
        assert!(parse(&[]).is_err());
        assert!(parse(&["frobnicate"]).is_err());
        assert!(parse(&["list", "monitors"]).is_err());
        assert!(parse(&["status", "--bogus"]).is_err());
        assert!(parse(&["switch"]).is_err(), "host is required");
    }

    #[test]
    fn init_host_specs() {
        let Command::Init(a) = parse(&[
            "init", "--host", "PC=hdmi1", "--host", "Laptop", "--host", "X=27",
        ])
        .unwrap()
        .command
        else {
            panic!()
        };
        let got: Vec<_> = a
            .hosts
            .iter()
            .map(|h| (h.label.as_str(), h.input.map(|i| i.0)))
            .collect();
        assert_eq!(got, [("PC", Some(17)), ("Laptop", None), ("X", Some(27))]);
        assert!(!a.write);
    }

    #[test]
    fn init_rejects_bad_input_and_force_without_write() {
        assert!(parse(&["init", "--host", "PC=dp9"]).is_err());
        assert!(parse(&["init", "--host", "=17"]).is_err());
        assert!(parse(&["init", "--force"]).is_err());
        assert!(parse(&["init", "--write", "--force"]).is_ok());
    }

    #[test]
    fn link_needs_two_ids() {
        let Command::Link { device, also } = parse(&["link", "a", "b"]).unwrap().command else {
            panic!()
        };
        assert_eq!((device.as_str(), also.as_str()), ("a", "b"));
        assert!(parse(&["link", "a"]).is_err());
    }

    #[test]
    fn show_takes_no_arguments() {
        assert!(matches!(parse(&["show"]).unwrap().command, Command::Show));
        assert!(parse(&["show", "extra"]).is_err());
    }

    #[test]
    fn list_targets() {
        let Command::List { what, ports } = parse(&["list", "devices"]).unwrap().command else {
            panic!()
        };
        assert_eq!((what, ports), (Some(ListTarget::Devices), false));
        let Command::List { what, ports } = parse(&["list"]).unwrap().command else {
            panic!()
        };
        assert_eq!((what, ports), (None, false));
        let Command::List { what, ports } =
            parse(&["list", "displays", "--ports"]).unwrap().command
        else {
            panic!()
        };
        assert_eq!((what, ports), (Some(ListTarget::Displays), true));
    }
}
