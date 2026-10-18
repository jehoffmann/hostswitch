// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hostswitch_core::{
    HostswitchError, Peripheral, PeripheralBackend, PeripheralKind, PeripheralTransport, Result,
    Slot,
};

use crate::hid_enum;
use crate::hidpp;
use crate::tool_runner::{ToolExecuting, ToolResult};

const HIDPP_VENDOR_ID: u16 = 0x046D;
/// USB receiver (Unifying/Bolt): vendor collection, short reports on usage 0x0001 (verified on hardware).
const VENDOR_USAGE_PAGE: u16 = 0xFF00;
const SHORT_REPORT_USAGE: u16 = 0x0001;
/// Bluetooth LE device: vendor collection for HID++ long reports (verified on hardware: ERGO K860).
const BLE_USAGE_PAGE: u16 = 0xFF43;
const BLE_USAGE: u16 = 0x0202;
/// Device index of a directly connected (Bluetooth) device; verified on hardware.
const BLE_DEVICE_INDEX: u8 = 0xFF;
/// Receiver-paired devices use 1...6.
const DEVICE_INDEXES: std::ops::RangeInclusive<u8> = 1..=6;
/// A Bluetooth device also emits unrelated input reports (mouse movement, keys), so read several reports
/// and pick the one that answers the request (`hidpp::find_reply`). Verified on hardware: a moving MX Vertical
/// delivered the reply as the 3rd report. hidapitester always performs every read, so each idle read costs its
/// timeout: 5 reads x 400 ms stays inside the 3 s tool timeout. A native hidapi backend can stop at the match.
const BLE_READS: u32 = 5;
const BLE_READ_TIMEOUT_MS: u32 = 400;
const RECEIVER_READ_TIMEOUT_MS: u32 = 800;

/// How a device is reached. Receiver devices share the receiver's vendor collection and are told apart by
/// device index 1...6; Bluetooth devices have their own collection and index 0xFF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    Receiver,
    Ble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Target {
    vendor_id: u16,
    product_id: u16,
    index: u8,
}

impl Target {
    fn channel(self) -> Channel {
        if self.index == BLE_DEVICE_INDEX {
            Channel::Ble
        } else {
            Channel::Receiver
        }
    }
}

/// A discoverable endpoint found by enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Found {
    Receiver {
        vendor_id: u16,
        product_id: u16,
    },
    Ble {
        vendor_id: u16,
        product_id: u16,
        name: String,
        kind: PeripheralKind,
    },
}

/// Keyboard/mouse backend that shells out to `hidapitester`. Contains command construction and sequencing only;
/// framing/parsing lives in `hidpp` / `hid_enum`.
///
/// Device id format: "<vid>:<pid>/<deviceIndex>" e.g. "046d:c52b/1" (receiver slot 1) or
/// "046d:b359/255" (Bluetooth device, index 0xFF).
/// TODO: two identical receivers / identical Bluetooth models cannot be told apart (no serial).
/// Bluetooth access needs macOS Input Monitoring for the process; without it, listing fails with
/// `PermissionDenied` (unless a receiver device was found).
pub struct HidapiTesterPeripheralBackend {
    runner: Arc<dyn ToolExecuting>,
    timeout: Duration,
}

impl HidapiTesterPeripheralBackend {
    pub fn new(runner: Arc<dyn ToolExecuting>) -> Self {
        Self {
            runner,
            timeout: Duration::from_secs(3),
        }
    }

    // ---- command construction ----

    fn enumerate_arguments() -> Vec<String> {
        ["--vidpid", "046D", "--list-detail"]
            .map(String::from)
            .to_vec()
    }

    /// `replies` is the number of `--read-input` to issue (0 for a write that gets no answer).
    fn exchange_arguments(
        channel: Channel,
        vendor_id: u16,
        product_id: u16,
        request: &[u8],
        replies: u32,
    ) -> Vec<String> {
        let (usage_page, usage, length, payload, read_timeout) = match channel {
            Channel::Receiver => (
                VENDOR_USAGE_PAGE,
                SHORT_REPORT_USAGE,
                hidpp::SHORT_REPORT_LENGTH,
                hidpp::short_form(request),
                RECEIVER_READ_TIMEOUT_MS,
            ),
            Channel::Ble => (
                BLE_USAGE_PAGE,
                BLE_USAGE,
                hidpp::LONG_REPORT_LENGTH,
                request.to_vec(),
                BLE_READ_TIMEOUT_MS,
            ),
        };
        let mut args: Vec<String> = vec![
            "--vidpid".into(),
            format!("{vendor_id:04X}/{product_id:04X}"),
            "--usagePage".into(),
            format!("0x{usage_page:04X}"),
            "--usage".into(),
            format!("0x{usage:04X}"),
            "--nonexclusive".into(),
            "--open".into(),
            "--length".into(),
            length.to_string(),
            "--timeout".into(),
            read_timeout.to_string(),
            "--send-output".into(),
            hidpp::datalist(&payload),
        ];
        args.extend((0..replies).map(|_| "--read-input".to_string()));
        args
    }

    pub fn make_device_id(vendor_id: u16, product_id: u16, index: u8) -> String {
        format!("{vendor_id:04x}:{product_id:04x}/{index}")
    }

    fn parse_device_id(id: &str) -> Option<Target> {
        let (ids, index) = id.split_once('/')?;
        let (vendor, product) = ids.split_once(':')?;
        Some(Target {
            vendor_id: u16::from_str_radix(vendor, 16).ok()?,
            product_id: u16::from_str_radix(product, 16).ok()?,
            index: index.parse().ok()?,
        })
    }

    /// Receiver devices have no real name yet (TODO: DEVICE_NAME_TYPE query), so name them by receiver type.
    fn receiver_device_name(product_id: u16, index: u8) -> String {
        match Self::transport_for(product_id) {
            PeripheralTransport::Bolt => format!("Bolt device {index}"),
            PeripheralTransport::Unifying => format!("Unifying device {index}"),
            _ => format!("Receiver {product_id:04X} device {index}"),
        }
    }

    fn transport_for(product_id: u16) -> PeripheralTransport {
        match product_id {
            0xC548 => PeripheralTransport::Bolt,
            0xC52B | 0xC532 => PeripheralTransport::Unifying,
            _ => PeripheralTransport::Unknown,
        }
    }

    // ---- helpers ----

    fn target(id: &str) -> Result<Target> {
        Self::parse_device_id(id)
            .ok_or_else(|| HostswitchError::InvalidConfig(format!("bad device id '{id}'")))
    }

    fn check_exit(result: &ToolResult) -> Result<()> {
        if result.exit_code == 0 {
            return Ok(());
        }
        // hidapitester prints its errors on stdout (verified), other tools' failures may use stderr: look at both.
        let text = Self::output_text(result);
        if text.contains("not permitted") {
            return Err(HostswitchError::PermissionDenied(
                "macOS blocked access to the device. Grant Input Monitoring to this app or your \
                 terminal (System Settings > Privacy & Security > Input Monitoring) and restart it"
                    .into(),
            ));
        }
        Err(HostswitchError::ToolFailed(format!(
            "hidapitester exit {}: {}",
            result.exit_code,
            Self::failure_detail(result)
        )))
    }

    fn output_text(result: &ToolResult) -> String {
        format!("{}\n{}", result.stdout, result.stderr)
    }

    /// stderr if there is any, else the `Error:` lines hidapitester printed on stdout, else all of stdout.
    fn failure_detail(result: &ToolResult) -> String {
        if !result.stderr.trim().is_empty() {
            return result.stderr.trim().to_string();
        }
        let errors: Vec<&str> = result
            .stdout
            .lines()
            .filter(|l| l.contains("Error"))
            .map(str::trim)
            .collect();
        if errors.is_empty() {
            result.stdout.trim().to_string()
        } else {
            errors.join("; ")
        }
    }

    /// Like `check_exit`, for talking to one device: if the device is not connected to this computer at all
    /// (a Bluetooth device that moved to another host disappears from the system), that is "offline" (F-13),
    /// so the executor can try another way to reach it (`deviceAliases`).
    fn check_exit_device(result: &ToolResult) -> Result<()> {
        if result.exit_code != 0 && Self::output_text(result).contains("no HID devices found") {
            return Err(HostswitchError::DeviceOffline(
                "device is not connected to this computer".into(),
            ));
        }
        Self::check_exit(result)
    }

    /// Receivers (USB) and Bluetooth devices found by enumeration.
    async fn discover(&self) -> Result<Vec<Found>> {
        let result = self
            .runner
            .run("hidapitester", &Self::enumerate_arguments(), self.timeout)
            .await?;
        Self::check_exit(&result)?;
        let endpoints: Vec<_> = hid_enum::parse(&result.stdout)
            .into_iter()
            .filter(|e| e.vendor_id == HIDPP_VENDOR_ID)
            .collect();

        let mut found = Vec::new();
        let mut seen = HashSet::new();
        for e in endpoints.iter().filter(|e| {
            !e.is_bluetooth() && e.usage_page == VENDOR_USAGE_PAGE && e.usage == SHORT_REPORT_USAGE
        }) {
            if seen.insert(e.product_id) {
                found.push(Found::Receiver {
                    vendor_id: e.vendor_id,
                    product_id: e.product_id,
                });
            }
        }

        // One Bluetooth device exposes several collections; group them by product id.
        let mut ble: BTreeMap<u16, Vec<&hid_enum::Endpoint>> = BTreeMap::new();
        for e in endpoints.iter().filter(|e| e.is_bluetooth()) {
            ble.entry(e.product_id).or_default().push(e);
        }
        for (product_id, group) in ble {
            if !group
                .iter()
                .any(|e| e.usage_page == BLE_USAGE_PAGE && e.usage == BLE_USAGE)
            {
                continue; // no HID++ collection
            }
            let name = group[0]
                .description
                .split_once(" - ") // hidapi reports "<manufacturer> - <product>"
                .map_or(group[0].description.as_str(), |(_, product)| product)
                .trim()
                .to_string();
            // Generic Desktop keyboard (0x01/0x06) means keyboard; else a pointer (0x01/0x02) means mouse.
            let has = |usage: u16| {
                group
                    .iter()
                    .any(|e| e.usage_page == 0x0001 && e.usage == usage)
            };
            let kind = if has(0x0006) {
                PeripheralKind::Keyboard
            } else if has(0x0002) {
                PeripheralKind::Mouse
            } else {
                PeripheralKind::Unknown
            };
            found.push(Found::Ble {
                vendor_id: HIDPP_VENDOR_ID,
                product_id,
                name: if name.is_empty() {
                    format!("Bluetooth device {product_id:04X}")
                } else {
                    name
                },
                kind,
            });
        }
        Ok(found)
    }

    async fn exchange(&self, t: Target, request: &[u8]) -> Result<Vec<u8>> {
        let replies = match t.channel() {
            Channel::Receiver => 1,
            Channel::Ble => BLE_READS,
        };
        let args =
            Self::exchange_arguments(t.channel(), t.vendor_id, t.product_id, request, replies);
        let result = self.runner.run("hidapitester", &args, self.timeout).await?;
        Self::check_exit_device(&result)?;
        let reports = hidpp::parse_read_outputs(&result.stdout);
        hidpp::find_reply(&reports, request)
            .cloned()
            .ok_or_else(|| {
                HostswitchError::ParseFailure("no reply report in hidapitester output".into())
            })
    }

    /// Like `exchange`, but maps "no reply" and the receiver's resource error (HID++ 1.0 code 09, which
    /// means empty slot OR asleep/elsewhere, see docs/research/hidpp.md) to `DeviceOffline` (F-13).
    async fn exchange_with_device(&self, t: Target, request: &[u8]) -> Result<Vec<u8>> {
        let bytes = match self.exchange(t, request).await {
            Err(HostswitchError::ParseFailure(_)) => {
                return Err(HostswitchError::DeviceOffline(
                    "no reply from device".into(),
                ));
            }
            other => other?,
        };
        if hidpp::is_resource_error(&bytes) {
            return Err(HostswitchError::DeviceOffline(
                "receiver reported resource error 09".into(),
            ));
        }
        Ok(bytes)
    }

    async fn change_host_feature_index(&self, t: Target) -> Result<u8> {
        let request = hidpp::get_feature(t.index, hidpp::CHANGE_HOST_FEATURE_ID);
        let bytes = self.exchange_with_device(t, &request).await?;
        hidpp::parse_feature_index(&bytes)?.ok_or_else(|| {
            HostswitchError::BackendUnavailable(
                "device does not support CHANGE_HOST (0x1814)".into(),
            )
        })
    }

    /// A device counts as present only if it echoes our ping. An empty receiver slot answers with a
    /// HID++ 1.0 error; a sleeping device may stay silent on the first request, so silence is retried once.
    /// Only a permission error is propagated; other failures mean "not present".
    async fn responds(&self, t: Target) -> Result<bool> {
        for _ in 0..2 {
            match self.exchange(t, &hidpp::ping(t.index)).await {
                Ok(bytes) => {
                    if hidpp::is_ping_reply(&bytes, t.index) {
                        return Ok(true);
                    }
                    if bytes.get(2) == Some(&0x8F) {
                        return Ok(false);
                    }
                }
                Err(e @ HostswitchError::PermissionDenied(_)) => return Err(e),
                Err(_) => {}
            }
        }
        Ok(false)
    }
}

#[async_trait]
impl PeripheralBackend for HidapiTesterPeripheralBackend {
    fn id(&self) -> &str {
        "hidapitester"
    }

    async fn is_available(&self) -> bool {
        self.runner.resolve("hidapitester").await.is_some()
    }

    async fn list_devices(&self) -> Result<Vec<Peripheral>> {
        let mut devices = Vec::new();
        let mut denied = None;
        for found in self.discover().await? {
            match found {
                Found::Receiver {
                    vendor_id,
                    product_id,
                } => {
                    for index in DEVICE_INDEXES {
                        let t = Target {
                            vendor_id,
                            product_id,
                            index,
                        };
                        match self.responds(t).await {
                            // TODO: query DEVICE_NAME_TYPE (0x0005) for the real name and kind.
                            Ok(true) => devices.push(Peripheral::new(
                                Self::make_device_id(vendor_id, product_id, index),
                                Self::receiver_device_name(product_id, index),
                                PeripheralKind::Unknown,
                                Self::transport_for(product_id),
                            )),
                            Ok(false) => {}
                            Err(e) => denied = Some(e),
                        }
                    }
                }
                Found::Ble {
                    vendor_id,
                    product_id,
                    name,
                    kind,
                } => {
                    let t = Target {
                        vendor_id,
                        product_id,
                        index: BLE_DEVICE_INDEX,
                    };
                    match self.responds(t).await {
                        Ok(true) => devices.push(Peripheral::new(
                            Self::make_device_id(vendor_id, product_id, BLE_DEVICE_INDEX),
                            name,
                            kind,
                            PeripheralTransport::Ble,
                        )),
                        Ok(false) => {}
                        Err(e) => denied = Some(e),
                    }
                }
            }
        }
        match denied {
            Some(e) if devices.is_empty() => Err(e),
            _ => Ok(devices),
        }
    }

    async fn current_host(&self, device: &str) -> Result<Option<Slot>> {
        let t = Self::target(device)?;
        let feature = self.change_host_feature_index(t).await?;
        let bytes = self
            .exchange_with_device(t, &hidpp::get_host_info(t.index, feature))
            .await?;
        Ok(hidpp::parse_host_info(&bytes)?.current_slot)
    }

    async fn switch_host(&self, slot: Slot, device: &str) -> Result<()> {
        let t = Self::target(device)?;
        let feature = self.change_host_feature_index(t).await?;
        let request = hidpp::set_current_host(t.index, feature, slot);
        // The device drops the link while switching, so no reply is awaited.
        let args = Self::exchange_arguments(t.channel(), t.vendor_id, t.product_id, &request, 0);
        let result = self.runner.run("hidapitester", &args, self.timeout).await?;
        Self::check_exit_device(&result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::FakeRunner;

    const ENUM: &str = include_str!("../tests/fixtures/hidapitester-list-detail-receiver.txt");
    const PING_LIVE: &str = include_str!("../tests/fixtures/hidpp-ping-live.txt");
    const PING_TIMEOUT: &str = include_str!("../tests/fixtures/hidpp-ping-timeout.txt");
    const GET_FEATURE: &str = include_str!("../tests/fixtures/hidpp-getfeature-changehost.txt");
    const HOST_INFO_ERR: &str =
        include_str!("../tests/fixtures/hidpp-gethostinfo-resource-error.txt");

    fn backend(runner: FakeRunner) -> (HidapiTesterPeripheralBackend, Arc<FakeRunner>) {
        let runner = Arc::new(runner);
        (HidapiTesterPeripheralBackend::new(runner.clone()), runner)
    }

    /// The receiver's reply for an empty slot `i` (the real fixture is for slot 6).
    fn empty_slot(i: u8) -> String {
        format!("read 7 bytes:\n 10 {i:02X} 8F 00 1A 09 00\n")
    }

    fn slot(n: u8) -> Slot {
        Slot::new(n).unwrap()
    }

    #[test]
    fn device_ids_round_trip_and_reject_garbage() {
        let id = HidapiTesterPeripheralBackend::make_device_id(0x046D, 0xC52B, 1);
        assert_eq!(id, "046d:c52b/1");
        let t = HidapiTesterPeripheralBackend::parse_device_id(&id).unwrap();
        assert_eq!((t.vendor_id, t.product_id, t.index), (0x046D, 0xC52B, 1));
        for bad in [
            "",
            "x",
            "046d:c52b",
            "046d/1",
            "zz:yy/1",
            "046d:c52b/300",
            "046d:c52b/a",
        ] {
            assert!(
                HidapiTesterPeripheralBackend::parse_device_id(bad).is_none(),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn exchange_arguments_match_the_hardware_verified_command() {
        let args = HidapiTesterPeripheralBackend::exchange_arguments(
            Channel::Receiver,
            0x046D,
            0xC52B,
            &hidpp::ping(1),
            1,
        );
        let expected = "--vidpid 046D/C52B --usagePage 0xFF00 --usage 0x0001 --nonexclusive --open --length 7 \
                    --timeout 800 --send-output 0x10,0x01,0x00,0x1A,0x00,0x00,0x5A --read-input";
        assert_eq!(args.join(" "), expected);
        let no_reply = HidapiTesterPeripheralBackend::exchange_arguments(
            Channel::Receiver,
            0x046D,
            0xC52B,
            &hidpp::ping(1),
            0,
        );
        assert!(!no_reply.contains(&"--read-input".to_string()));
    }

    #[test]
    fn ble_arguments_use_the_long_vendor_collection_and_several_reads() {
        let args = HidapiTesterPeripheralBackend::exchange_arguments(
            Channel::Ble,
            0x046D,
            0xB359,
            &hidpp::ping(0xFF),
            BLE_READS,
        );
        let expected = "--vidpid 046D/B359 --usagePage 0xFF43 --usage 0x0202 --nonexclusive --open --length 20 \
                    --timeout 400 --send-output 0x11,0xFF,0x00,0x1A,0x00,0x00,0x5A,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00 \
                    --read-input --read-input --read-input --read-input --read-input";
        assert_eq!(
            args.join(" "),
            expected,
            "matches the command verified on the K860"
        );
    }

    #[test]
    fn channel_follows_the_device_index() {
        let id = |s: &str| {
            HidapiTesterPeripheralBackend::parse_device_id(s)
                .unwrap()
                .channel()
        };
        assert_eq!(id("046d:c52b/1"), Channel::Receiver);
        assert_eq!(id("046d:c52b/6"), Channel::Receiver);
        assert_eq!(id("046d:b359/255"), Channel::Ble);
    }

    #[tokio::test]
    async fn bad_device_id_makes_no_tool_call() {
        let (b, runner) = backend(FakeRunner::available());
        assert!(matches!(
            b.switch_host(slot(1), "garbage").await,
            Err(HostswitchError::InvalidConfig(_))
        ));
        assert!(runner.recorded().is_empty());
    }

    #[tokio::test]
    async fn discovery_lists_only_genuine_ping_echoes() {
        // enumeration, then per index 1..=6: live, then empty-slot errors (silence on 2 would be retried).
        let mut runner = FakeRunner::available().reply(ENUM, 0).reply(PING_LIVE, 0);
        for i in 2..=6 {
            runner = runner.reply(&empty_slot(i), 0);
        }
        let (b, _) = backend(runner);
        let devices = b.list_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "046d:c52b/1");
        assert_eq!(devices[0].transport, PeripheralTransport::Unifying);
        assert!(devices[0].is_online());
    }

    #[tokio::test]
    async fn silence_is_retried_once_before_giving_up_on_a_slot() {
        let (b, runner) = backend(
            FakeRunner::available()
                .reply(PING_TIMEOUT, 0)
                .reply(PING_LIVE, 0),
        );
        let t = Target {
            vendor_id: 0x046D,
            product_id: 0xC52B,
            index: 1,
        };
        assert!(b.responds(t).await.unwrap());
        assert_eq!(runner.recorded().len(), 2);
    }

    #[tokio::test]
    async fn switch_sends_the_write_only_after_a_successful_feature_lookup() {
        let (b, runner) = backend(FakeRunner::available().reply(GET_FEATURE, 0).reply("", 0));
        b.switch_host(slot(2), "046d:c52b/1").await.unwrap();
        let calls = runner.recorded();
        assert_eq!(calls.len(), 2);
        let write = calls[1].1.join(" ");
        assert!(
            write.contains("--send-output 0x10,0x01,0x09,0x1A,0x01,0x00,0x00"),
            "{write}"
        );
        assert!(
            !write.contains("--read-input"),
            "no reply is awaited for the write"
        );
    }

    #[tokio::test]
    async fn switch_never_writes_when_the_lookup_fails() {
        // resource error, silence, unparseable output, unsupported feature, and a HID++ 2.0 error report
        let unsupported = "read 7 bytes:\n 11 01 00 0A 00 00 00\n";
        let hidpp2_error = "read 7 bytes:\n 11 01 FF 00 0A 02 00\n";
        for reply in [
            HOST_INFO_ERR,
            PING_TIMEOUT,
            "garbage",
            unsupported,
            hidpp2_error,
        ] {
            let (b, runner) = backend(FakeRunner::available().reply(reply, 0).reply("", 0));
            assert!(
                b.switch_host(slot(1), "046d:c52b/1").await.is_err(),
                "{reply:?}"
            );
            assert_eq!(
                runner.recorded().len(),
                1,
                "only the lookup was sent for {reply:?}"
            );
        }
    }

    #[tokio::test]
    async fn resource_error_and_silence_map_to_device_offline() {
        for reply in [HOST_INFO_ERR, PING_TIMEOUT] {
            let (b, _) = backend(FakeRunner::available().reply(reply, 0));
            let err = b.switch_host(slot(1), "046d:c52b/1").await.unwrap_err();
            assert!(matches!(err, HostswitchError::DeviceOffline(_)), "{err:?}");
            assert_eq!(err.to_string(), "device offline");
        }
    }

    #[tokio::test]
    async fn current_host_never_sends_a_write_frame() {
        let host_info = "read 7 bytes:\n 11 01 09 0A 03 01 00\n";
        let (b, runner) = backend(
            FakeRunner::available()
                .reply(GET_FEATURE, 0)
                .reply(host_info, 0),
        );
        assert_eq!(b.current_host("046d:c52b/1").await.unwrap(), Some(slot(2)));
        for (_, args) in runner.recorded() {
            let sent = args.join(" ");
            assert!(
                !sent.contains("0x1A,0x01") && !sent.contains("0x10,0x01,0x09,0x1A"),
                "{sent}"
            );
        }
    }

    #[tokio::test]
    async fn tool_failure_is_reported_not_swallowed() {
        let (b, _) = backend(FakeRunner::available().reply_err("boom", 4));
        let err = b.switch_host(slot(1), "046d:c52b/1").await.unwrap_err();
        assert_eq!(
            err,
            HostswitchError::ToolFailed("hidapitester exit 4: boom".into())
        );
    }

    const ENUM_BLE: &str = include_str!("../tests/fixtures/hidapitester-list-detail-ble.txt");
    const BLE_PING: &str = include_str!("../tests/fixtures/hidpp-ble-ping-live.txt");
    const BLE_FEATURE: &str = include_str!("../tests/fixtures/hidpp-ble-getfeature-changehost.txt");
    const BLE_NOISE: &str = include_str!("../tests/fixtures/hidpp-ble-mouse-noise.txt");

    #[tokio::test]
    async fn discovery_finds_bluetooth_devices_with_names_and_kinds() {
        // enumeration, then the MX Vertical (noise + ping reply), the K860, and 6 empty receiver slots.
        let noise_then_ping = "read 8 bytes:\n 02 00 00 FF 0F 00 00 00\nread 20 bytes:\n 11 FF 00 1A 04 05 5A 00 00 00 00 00 00 00 00 00 00 00 00 00\n";
        let mut runner = FakeRunner::available().reply(ENUM_BLE, 0);
        for i in 1..=6 {
            runner = runner.reply(&empty_slot(i), 0);
        }
        runner = runner.reply(noise_then_ping, 0).reply(BLE_PING, 0);
        let (b, runner) = backend(runner);
        let devices = b.list_devices().await.unwrap();

        assert_eq!(devices.len(), 2, "{devices:?}");
        let mx = &devices[0];
        assert_eq!(mx.id, "046d:b020/255");
        assert_eq!(mx.name, "MX Vertical");
        assert_eq!(mx.kind, PeripheralKind::Mouse);
        assert_eq!(mx.transport, PeripheralTransport::Ble);
        let k860 = &devices[1];
        assert_eq!(k860.id, "046d:b359/255");
        assert_eq!(k860.name, "ERGO K860");
        assert_eq!(k860.kind, PeripheralKind::Keyboard);
        // Bluetooth pings go to the vendor long collection with several reads; the receiver path is unchanged.
        let calls = runner.recorded();
        assert!(calls[1].1.join(" ").contains("--usagePage 0xFF00"));
        assert!(calls[7].1.join(" ").contains("--usagePage 0xFF43"));
        assert_eq!(
            calls[7].1.iter().filter(|a| *a == "--read-input").count(),
            BLE_READS as usize
        );
    }

    #[tokio::test]
    async fn bluetooth_switch_sends_a_long_report_to_index_ff_after_lookup() {
        let (b, runner) = backend(FakeRunner::available().reply(BLE_FEATURE, 0).reply("", 0));
        b.switch_host(slot(2), "046d:b359/255").await.unwrap();
        let calls = runner.recorded();
        let write = calls[1].1.join(" ");
        assert!(write.contains("--usagePage 0xFF43"), "{write}");
        assert!(write.contains("--length 20"), "{write}");
        assert!(
            write.contains("--send-output 0x11,0xFF,0x08,0x1A,0x01,"),
            "feature index 8, slot 2 -> wire 1: {write}"
        );
        assert!(!write.contains("--read-input"));
    }

    #[tokio::test]
    async fn reply_is_picked_out_of_unrelated_input_reports() {
        let (b, _) = backend(FakeRunner::available().reply(BLE_NOISE, 0).reply("", 0));
        b.switch_host(slot(1), "046d:b020/255").await.unwrap();
    }

    #[tokio::test]
    async fn only_noise_means_the_device_is_offline() {
        let only_noise = "read 8 bytes:\n 02 00 00 FF 0F 00 00 00\n";
        let (b, runner) = backend(FakeRunner::available().reply(only_noise, 0));
        let err = b.switch_host(slot(1), "046d:b020/255").await.unwrap_err();
        assert!(matches!(err, HostswitchError::DeviceOffline(_)), "{err:?}");
        assert_eq!(runner.recorded().len(), 1, "no write was sent");
    }

    #[tokio::test]
    async fn device_missing_from_the_system_is_offline_not_a_tool_failure() {
        // Seen on hardware after the keyboard switched to another host.
        let gone = "Opening device, vid/pid:0x046D/0xB359, usagePage/usage: FF43/202\nError: no HID devices found for given vid/pid\n";
        let (b, _) = backend(FakeRunner::available().reply(gone, 1));
        let err = b.switch_host(slot(1), "046d:b359/255").await.unwrap_err();
        assert!(matches!(err, HostswitchError::DeviceOffline(_)), "{err:?}");
        // Enumeration failures stay tool failures.
        let (b, _) = backend(
            FakeRunner::available().reply("Error: no HID devices found for given vid/pid", 1),
        );
        assert!(matches!(
            b.list_devices().await,
            Err(HostswitchError::ToolFailed(_))
        ));
    }

    #[tokio::test]
    async fn errors_on_stderr_are_recognised_too() {
        let (b, _) = backend(
            FakeRunner::available().reply_err("Error: no HID devices found for given vid/pid", 1),
        );
        let err = b.switch_host(slot(1), "046d:b359/255").await.unwrap_err();
        assert!(matches!(err, HostswitchError::DeviceOffline(_)), "{err:?}");
    }

    #[tokio::test]
    async fn tool_failure_message_shows_the_error_line_from_stdout() {
        let out = "Opening device, vid/pid:0x046D/0xC52B, usagePage/usage: FF00/1\nError: could not open device at path: x\n";
        let (b, _) = backend(FakeRunner::available().reply(out, 1));
        let err = b.switch_host(slot(1), "046d:c52b/1").await.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Tool failed: hidapitester exit 1: Error: could not open device at path: x"
        );
    }

    #[tokio::test]
    async fn input_monitoring_denial_is_reported_not_hidden() {
        let denied = "Error: hid_open_path: failed to open IOHIDDevice from mach entry: (0xE00002E2) (iokit/common) not permitted";
        // Only Bluetooth devices, all denied: the user must see why the list is empty.
        let ble_only = ENUM_BLE
            .split("\n\n")
            .filter(|b| b.contains("Bluetooth"))
            .collect::<Vec<_>>()
            .join("\n\n");
        let (b, _) = backend(
            FakeRunner::available()
                .reply(&ble_only, 0)
                .reply(denied, 1)
                .reply(denied, 1),
        );
        let err = b.list_devices().await.unwrap_err();
        assert!(
            matches!(err, HostswitchError::PermissionDenied(_)),
            "{err:?}"
        );
        assert!(err.to_string().contains("Input Monitoring"));
    }

    #[tokio::test]
    async fn receiver_devices_survive_a_bluetooth_permission_denial() {
        let denied = "Error: ... not permitted";
        let mut runner = FakeRunner::available()
            .reply(ENUM_BLE, 0)
            .reply(PING_LIVE, 0);
        for i in 2..=6 {
            runner = runner.reply(&empty_slot(i), 0);
        }
        runner = runner.reply(denied, 1).reply(denied, 1);
        let (b, _) = backend(runner);
        let devices = b.list_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "046d:c52b/1");
    }

    #[tokio::test]
    async fn enumeration_without_bluetooth_collections_finds_no_ble_device() {
        let no_hidpp = "046D/B359: Logitech - ERGO K860\n  vendorId: 0x046D\n  productId: 0xB359\n  usagePage: 0x0001\n  usage: 0x0006\n  interface: -1\n  bus_type: Bluetooth (2)\n  path: x\n";
        let (b, runner) = backend(FakeRunner::available().reply(no_hidpp, 0));
        assert!(b.list_devices().await.unwrap().is_empty());
        assert_eq!(runner.recorded().len(), 1);
    }
}
