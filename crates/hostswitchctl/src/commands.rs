// SPDX-License-Identifier: Apache-2.0

use hostswitch_backends::BackendRegistry;
use hostswitch_core::bootstrap::{Detected, DetectedDisplay, bootstrap};
use hostswitch_core::domain::{DeviceCatalog, PeripheralKind};
use hostswitch_core::{
    AppConfig, ConfigStore, ExecutorOptions, HostswitchError, ObservedState, SwitchExecutor, plan,
};

use crate::args::{Cli, Command, InitArgs, ListTarget, SwitchArgs};
use crate::render::{
    DisplayRow, PORTS_NOTE, format_device_list, format_display_list, format_mapping, format_plan,
    format_ports, format_report, format_status,
};

/// Exit codes: 0 ok, 1 error, 2 switch partly/fully failed, 64 usage (set in main).
pub async fn run(cli: Cli) -> u8 {
    let registry = BackendRegistry::default_backends();
    let store = cli.config.map(ConfigStore::new).unwrap_or_default();
    let result = match cli.command {
        Command::List { what, ports } => list(what, ports, &registry, &store).await,
        Command::Show => show(&store),
        Command::Link { device, also } => link(&store, &device, &also),
        Command::Status => status(&store, &registry).await,
        Command::Init(args) => init(&args, &store, &registry).await,
        Command::Switch(args) => switch_host(&args, &store, &registry).await,
    };
    result.unwrap_or_else(|e| {
        eprintln!("error: {e}");
        1
    })
}

async fn list(
    what: Option<ListTarget>,
    show_ports: bool,
    registry: &BackendRegistry,
    store: &ConfigStore,
) -> Result<u8, HostswitchError> {
    let mut failed = false;
    if matches!(what, None | Some(ListTarget::Displays)) {
        println!("Displays:");
        match registry.display().await {
            Some(backend) => match backend.list_displays().await {
                Ok(displays) => {
                    let mut rows = Vec::new();
                    let mut ports = Vec::new();
                    for d in displays {
                        let input = backend.current_input(&d.id).await.ok().flatten();
                        ports.push(input);
                        rows.push(DisplayRow {
                            name: d.name,
                            id: d.id,
                            input,
                        });
                    }
                    print!("{}", format_display_list(&rows));
                    if show_ports {
                        for (row, input) in rows.iter().zip(ports) {
                            println!("\n  {}", row.name);
                            print!("{}", format_ports(input));
                        }
                        if !rows.is_empty() {
                            println!("\n{PORTS_NOTE}");
                        }
                    }
                }
                Err(e) => {
                    println!("  error: {e}");
                    failed = true;
                }
            },
            None => {
                println!("  no display backend available (is m1ddc installed? Apple Silicon only)")
            }
        }
    }
    if matches!(what, None | Some(ListTarget::Devices)) {
        println!("Devices:");
        match registry.peripherals().await {
            Some(backend) => match backend.list_devices().await {
                Ok(live) => {
                    // F-13: remember what was seen (config only, never hardware) and show the rest as offline.
                    let mut config = store.load().unwrap_or_default();
                    if config.remember(&live) {
                        let _ = store.save(&config);
                    }
                    let devices = DeviceCatalog::merge(&live, &config.known_devices);
                    print!("{}", format_device_list(&devices));
                }
                Err(e) => {
                    println!("  error: {e}");
                    failed = true;
                }
            },
            None => println!("  no keyboard/mouse backend available (is hidapitester installed?)"),
        }
    }
    Ok(u8::from(failed))
}

async fn status(store: &ConfigStore, registry: &BackendRegistry) -> Result<u8, HostswitchError> {
    let config = store.load()?;
    let mut observed = ObservedState::default();
    let mut display_names = std::collections::HashMap::new();
    if let Some(backend) = registry.display().await {
        if let Ok(displays) = backend.list_displays().await {
            display_names.extend(displays.into_iter().map(|d| (d.id, d.name)));
        }
        for id in display_ids(&config) {
            if let Ok(Some(input)) = backend.current_input(&id).await {
                observed.display_inputs.insert(id, input);
            }
        }
    }
    if let Some(backend) = registry.peripherals().await {
        for id in device_ids(&config) {
            // Ask every way to reach the device; the first that answers wins.
            for candidate in config.device_candidates(&id) {
                if let Ok(Some(slot)) = backend.current_host(&candidate).await {
                    observed.device_slots.insert(id.clone(), slot);
                    break;
                }
            }
        }
    }
    print!("{}", format_status(&config, &display_names, &observed));
    Ok(0)
}

fn link(store: &ConfigStore, device: &str, also: &str) -> Result<u8, HostswitchError> {
    let mut config = store.load()?;
    config.link_device(device, also)?;
    store.save(&config)?;
    println!("{device} can now also be reached as {also}");
    Ok(0)
}

/// Prints the mapping from the config file only; never touches hardware.
fn show(store: &ConfigStore) -> Result<u8, HostswitchError> {
    let config = store.load()?;
    println!("Config: {}\n", store.path.display());
    print!("{}", format_mapping(&config));
    Ok(0)
}

/// Read-only detection, then `bootstrap` builds the config. Prints the JSON; `--write` saves it.
async fn init(
    args: &InitArgs,
    store: &ConfigStore,
    registry: &BackendRegistry,
) -> Result<u8, HostswitchError> {
    let existing = match store.load() {
        Ok(config) => config,
        Err(e) if args.write && !args.force => return Err(e),
        Err(_) => AppConfig::default(),
    };
    if args.write && !args.force && !existing.hosts.is_empty() {
        return Err(HostswitchError::InvalidConfig(format!(
            "{} already has hosts; use --force to replace them (remembered devices are kept)",
            store.path.display()
        )));
    }

    let mut detected = Detected::default();
    match registry.display().await {
        Some(backend) => match backend.list_displays().await {
            Ok(displays) => {
                for display in displays {
                    let input = backend.current_input(&display.id).await.ok().flatten();
                    detected.displays.push(DetectedDisplay { display, input });
                }
            }
            Err(e) => eprintln!("warning: displays: {e}"),
        },
        None => eprintln!(
            "warning: no display backend available (is m1ddc installed? Apple Silicon only)"
        ),
    }
    match registry.peripherals().await {
        Some(backend) => match backend.list_devices().await {
            Ok(devices) => {
                for d in devices.iter().filter(|d| {
                    d.is_online()
                        && matches!(d.kind, PeripheralKind::Keyboard | PeripheralKind::Mouse)
                }) {
                    if let Ok(Some(slot)) = backend.current_host(&d.id).await {
                        detected.slots.insert(d.id.clone(), slot);
                    }
                }
                detected.devices = devices;
            }
            Err(e) => eprintln!("warning: keyboard/mouse devices: {e}"),
        },
        None => {
            eprintln!("warning: no keyboard/mouse backend available (is hidapitester installed?)")
        }
    }

    let hostname = this_hostname();
    let label = args
        .label
        .clone()
        .unwrap_or_else(|| hostname.split('.').next().unwrap_or(&hostname).to_string());
    let built = bootstrap(
        &label,
        &hostname,
        &args.hosts,
        &detected,
        &existing.known_devices,
    )?;
    for note in &built.notes {
        eprintln!("note: {note}");
    }
    if args.write {
        store.save(&built.config)?;
        println!("Wrote {}", store.path.display());
    } else {
        let json = ConfigStore::encode(&built.config)?;
        println!("{}", String::from_utf8_lossy(&json));
        eprintln!(
            "Nothing written. Re-run with --write to save to {}",
            store.path.display()
        );
    }
    Ok(0)
}

fn this_hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "this-computer".to_string())
}

fn display_ids(config: &AppConfig) -> std::collections::BTreeSet<String> {
    config
        .profiles
        .iter()
        .flat_map(|p| p.displays.iter().map(|d| d.display_id.clone()))
        .collect()
}

fn device_ids(config: &AppConfig) -> std::collections::BTreeSet<String> {
    config
        .profiles
        .iter()
        .flat_map(|p| {
            [&p.keyboard, &p.mouse]
                .into_iter()
                .flatten()
                .map(|a| a.device_id.clone())
        })
        .collect()
}

async fn switch_host(
    args: &SwitchArgs,
    store: &ConfigStore,
    registry: &BackendRegistry,
) -> Result<u8, HostswitchError> {
    let config = store.load()?;
    let host = config
        .host_matching(&args.host)
        .ok_or_else(|| HostswitchError::UnknownHost(args.host.clone()))?;
    let profile = config.profile(host.id).ok_or_else(|| {
        HostswitchError::InvalidConfig(format!("no profile for host '{}'", host.label))
    })?;
    let steps = plan(profile);
    print!("{}", format_plan(&host.label, &steps, &config));
    if args.dry_run {
        println!("Dry run: nothing was changed.");
        return Ok(0);
    }
    if steps.is_empty() {
        println!("Nothing to switch: the profile has no displays or devices.");
        return Ok(0);
    }
    let executor = SwitchExecutor::new(
        registry.display().await,
        registry.peripherals().await,
        ExecutorOptions::default(),
    )
    .with_aliases(config.device_aliases.clone());
    let report = executor.execute(&steps).await;
    print!("{}", format_report(&report, &config));
    Ok(if report.is_fully_successful() { 0 } else { 2 })
}
