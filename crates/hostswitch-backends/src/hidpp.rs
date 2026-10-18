// SPDX-License-Identifier: Apache-2.0

//! HID++ 2.0 framing and pure response parsers.
//! Layouts come from docs/research/hidpp.md. UNVERIFIED on hardware: the getHostInfo reply layout and
//! the BLE device index. Keep those assumptions isolated here.

use hostswitch_core::{HostswitchError, Result, Slot};

pub const SHORT_REPORT_ID: u8 = 0x10;
pub const SHORT_REPORT_LENGTH: usize = 7;
pub const LONG_REPORT_ID: u8 = 0x11;
pub const LONG_REPORT_LENGTH: usize = 20;
pub const VERY_LONG_REPORT_ID: u8 = 0x12;
pub const ROOT_FEATURE_INDEX: u8 = 0x00;
pub const CHANGE_HOST_FEATURE_ID: u16 = 0x1814;
/// Nonzero software id so replies are distinguishable from device notifications.
pub const SOFTWARE_ID: u8 = 0x0A;
const PING_BYTE: u8 = 0x5A;
const ERROR_HIDPP1: u8 = 0x8F;
const ERROR_HIDPP2: u8 = 0xFF;

/// Builds a long (20 byte) request: `[reportId, deviceIndex, featureIndex, (function<<4)|swId, params...]`.
pub fn request(device_index: u8, feature_index: u8, function: u8, params: &[u8]) -> Vec<u8> {
    let mut bytes = vec![
        LONG_REPORT_ID,
        device_index,
        feature_index,
        (function << 4) | SOFTWARE_ID,
    ];
    bytes.extend_from_slice(params);
    bytes.resize(LONG_REPORT_LENGTH, 0);
    bytes
}

/// Root.getFeature(featureID) -> reply param[0] is the feature index (0 = unsupported).
pub fn get_feature(device_index: u8, feature_id: u16) -> Vec<u8> {
    request(
        device_index,
        ROOT_FEATURE_INDEX,
        0,
        &feature_id.to_be_bytes(),
    )
}

/// Root.getProtocolVersion used as a ping; the reply echoes the ping byte.
pub fn ping(device_index: u8) -> Vec<u8> {
    request(device_index, ROOT_FEATURE_INDEX, 1, &[0, 0, PING_BYTE])
}

/// CHANGE_HOST fn0, read-only.
pub fn get_host_info(device_index: u8, feature_index: u8) -> Vec<u8> {
    request(device_index, feature_index, 0, &[])
}

/// CHANGE_HOST fn1. WRITE: switches the device away from this host. The wire value is 0-based.
pub fn set_current_host(device_index: u8, feature_index: u8, slot: Slot) -> Vec<u8> {
    request(device_index, feature_index, 1, &[slot.get() - 1])
}

/// Short (0x10, 7 byte) form of a request; every request we send has at most 3 parameter bytes.
/// Verified on hardware: the receiver answers short requests on usage 0x0001 (the reply may be a 0x11 report).
pub fn short_form(request: &[u8]) -> Vec<u8> {
    if request.len() < SHORT_REPORT_LENGTH {
        return request.to_vec();
    }
    let mut out = vec![SHORT_REPORT_ID];
    out.extend_from_slice(&request[1..SHORT_REPORT_LENGTH]);
    out
}

/// Formats bytes for hidapitester's `--send-output` datalist.
pub fn datalist(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("0x{b:02X}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn is_report_id(id: u8) -> bool {
    matches!(id, SHORT_REPORT_ID | LONG_REPORT_ID | VERY_LONG_REPORT_ID)
}

/// Extracts the last input report from `hidapitester --read-input` output.
/// See `parse_read_outputs` for the format.
pub fn parse_read_output(output: &str) -> Option<Vec<u8>> {
    parse_read_outputs(output).pop()
}

/// Extracts every input report from `hidapitester` output (one block per `--read-input`).
/// Real format (fixtures hidpp-*.txt): "read N bytes:" followed by one line of space-separated hex.
/// A timeout prints "read 0 bytes:" plus a line of zeros; that is "no reply", not a report, and is skipped.
pub fn parse_read_outputs(output: &str) -> Vec<Vec<u8>> {
    let lines: Vec<&str> = output.lines().collect();
    let mut reports = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !(line.contains("read") && line.contains("bytes")) {
            continue;
        }
        let Some(count) = read_byte_count(line).filter(|&n| n > 0) else {
            continue;
        };
        let mut bytes = Vec::new();
        for next in &lines[i + 1..] {
            let tokens: Vec<&str> = next.split_whitespace().collect();
            let parsed: Vec<u8> = tokens
                .iter()
                .filter(|t| t.len() == 2)
                .filter_map(|t| u8::from_str_radix(t, 16).ok())
                .collect();
            if tokens.is_empty() || parsed.len() != tokens.len() {
                break;
            }
            bytes.extend(parsed);
        }
        bytes.truncate(count);
        if !bytes.is_empty() {
            reports.push(bytes);
        }
    }
    reports
}

/// The report that answers `request`, skipping unrelated input (mouse movement, key events and
/// notifications also arrive on the same device). A reply is a HID++ report for the same device index
/// whose feature index and function/software id echo the request, or a HID++ 1.0 / 2.0 error report.
pub fn find_reply<'a>(reports: &'a [Vec<u8>], request: &[u8]) -> Option<&'a Vec<u8>> {
    if request.len() < 4 {
        return None;
    }
    reports.iter().find(|r| {
        r.len() >= 7
            && is_report_id(r[0])
            && r[1] == request[1]
            && (r[2] == ERROR_HIDPP1
                || r[2] == ERROR_HIDPP2
                || (r[2] == request[2] && r[3] == request[3]))
    })
}

fn read_byte_count(line: &str) -> Option<usize> {
    line.match_indices("read ").find_map(|(i, m)| {
        let rest = &line[i + m.len()..];
        let digits_end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let (digits, tail) = rest.split_at(digits_end);
        if digits.is_empty() || !tail.starts_with(" bytes") {
            return None;
        }
        digits.parse().ok()
    })
}

/// True only for a genuine answer to `ping(device_index)`. HID++ 1.0 errors (empty slot) and
/// replies from another device index do not count.
pub fn is_ping_reply(bytes: &[u8], device_index: u8) -> bool {
    bytes.len() >= 7
        && is_report_id(bytes[0])
        && bytes[1] == device_index
        && bytes[2] == ROOT_FEATURE_INDEX
        && bytes[3] >> 4 == 1
        && bytes[6] == PING_BYTE
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub device_index: u8,
    pub feature_index: u8,
    pub function: u8,
    pub params: Vec<u8>,
}

pub fn parse_reply(bytes: &[u8]) -> Result<Reply> {
    if bytes.len() < 7 || !is_report_id(bytes[0]) {
        return Err(HostswitchError::ParseFailure(format!(
            "not a HID++ report: {:?}",
            &bytes[..bytes.len().min(4)]
        )));
    }
    match bytes[2] {
        // HID++ 1.0 error (e.g. empty receiver slot): [id, dev, 0x8F, subId, addr, errCode]
        ERROR_HIDPP1 => Err(HostswitchError::ToolFailed(format!(
            "HID++ 1.0 error code {}",
            bytes[5]
        ))),
        // HID++ 2.0 error report: [id, dev, 0xFF, featIdx, fn|sw, errCode]
        ERROR_HIDPP2 => Err(HostswitchError::ToolFailed(format!(
            "HID++ error code {}",
            bytes[5]
        ))),
        _ => Ok(Reply {
            device_index: bytes[1],
            feature_index: bytes[2],
            function: bytes[3] >> 4,
            params: bytes[4..].to_vec(),
        }),
    }
}

/// Feature index of the requested feature, or `None` if the device does not support it.
pub fn parse_feature_index(bytes: &[u8]) -> Result<Option<u8>> {
    let reply = parse_reply(bytes)?;
    let index = *reply
        .params
        .first()
        .ok_or_else(|| HostswitchError::ParseFailure("empty getFeature reply".into()))?;
    Ok((index != 0).then_some(index))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostInfo {
    pub host_count: u8,
    pub current_slot: Option<Slot>,
}

/// UNVERIFIED layout: params = [nbHosts, currentHost (0-based), flags...].
pub fn parse_host_info(bytes: &[u8]) -> Result<HostInfo> {
    let reply = parse_reply(bytes)?;
    if reply.params.len() < 2 {
        return Err(HostswitchError::ParseFailure(
            "short getHostInfo reply".into(),
        ));
    }
    Ok(HostInfo {
        host_count: reply.params[0],
        current_slot: reply.params[1].checked_add(1).and_then(Slot::new),
    })
}

/// True for the receiver's HID++ 1.0 resource error (09): empty slot OR asleep/elsewhere (R-6).
pub fn is_resource_error(bytes: &[u8]) -> bool {
    bytes.len() >= 6 && bytes[2] == ERROR_HIDPP1 && bytes[5] == 0x09
}

#[cfg(test)]
mod tests {
    use super::*;

    const PING_LIVE: &str = include_str!("../tests/fixtures/hidpp-ping-live.txt");
    const PING_EMPTY: &str = include_str!("../tests/fixtures/hidpp-ping-empty-slot.txt");
    const PING_TIMEOUT: &str = include_str!("../tests/fixtures/hidpp-ping-timeout.txt");
    const GET_FEATURE: &str = include_str!("../tests/fixtures/hidpp-getfeature-changehost.txt");
    const HOST_INFO_ERR: &str =
        include_str!("../tests/fixtures/hidpp-gethostinfo-resource-error.txt");

    #[test]
    fn requests_have_the_verified_layout() {
        let r = get_feature(1, 0x1814);
        assert_eq!(r.len(), LONG_REPORT_LENGTH);
        assert_eq!(&r[..6], [0x11, 0x01, 0x00, 0x0A, 0x18, 0x14]);
        assert_eq!(
            datalist(&short_form(&ping(6))),
            "0x10,0x06,0x00,0x1A,0x00,0x00,0x5A"
        );
        assert_eq!(
            datalist(&short_form(&get_feature(1, 0x1814))),
            "0x10,0x01,0x00,0x0A,0x18,0x14,0x00"
        );
    }

    #[test]
    fn set_current_host_uses_a_zero_based_slot() {
        for (slot, wire) in [(1, 0), (2, 1), (3, 2)] {
            let r = set_current_host(1, 9, Slot::new(slot).unwrap());
            assert_eq!(&r[..5], [0x11, 0x01, 0x09, 0x1A, wire]);
        }
    }

    #[test]
    fn parses_recorded_replies() {
        let live = parse_read_output(PING_LIVE).unwrap();
        assert_eq!(live, [0x11, 0x01, 0x00, 0x1A, 0x04, 0x05, 0x5A]);
        assert!(is_ping_reply(&live, 1));
        assert!(!is_ping_reply(&live, 2), "other device index");

        let empty = parse_read_output(PING_EMPTY).unwrap();
        assert!(!is_ping_reply(&empty, 6));
        assert!(is_resource_error(&empty));

        assert_eq!(
            parse_read_output(PING_TIMEOUT),
            None,
            "timeout is no reply, not a report"
        );

        let feature = parse_read_output(GET_FEATURE).unwrap();
        assert_eq!(parse_feature_index(&feature).unwrap(), Some(9));

        let err = parse_read_output(HOST_INFO_ERR).unwrap();
        assert_eq!(&err[..6], [0x10, 0x01, 0x8F, 0x09, 0x0A, 0x09]);
        assert!(is_resource_error(&err));
        assert_eq!(
            parse_host_info(&err),
            Err(HostswitchError::ToolFailed("HID++ 1.0 error code 9".into()))
        );
    }

    #[test]
    fn read_output_parser_handles_garbage_and_truncation() {
        for bad in [
            "",
            "nothing here",
            "read bytes:\n 11 22",
            "read 7 bytes:\n zz yy",
            "read 0 bytes:\n 00 00",
        ] {
            assert_eq!(parse_read_output(bad), None, "{bad:?}");
        }
        assert_eq!(
            parse_read_output("read 3 bytes:\n 10 20 30 40 50"),
            Some(vec![0x10, 0x20, 0x30])
        );
        assert_eq!(
            parse_read_output("read 2 bytes:\r\n 10 20\r\n"),
            Some(vec![0x10, 0x20])
        );
        assert_eq!(
            parse_read_output("read 1 bytes:\n 01\nread 2 bytes:\n 0A 0B"),
            Some(vec![0x0A, 0x0B]),
            "last report wins"
        );
    }

    const BLE_PING: &str = include_str!("../tests/fixtures/hidpp-ble-ping-live.txt");
    const BLE_FEATURE: &str = include_str!("../tests/fixtures/hidpp-ble-getfeature-changehost.txt");
    const BLE_NOISE: &str = include_str!("../tests/fixtures/hidpp-ble-mouse-noise.txt");

    #[test]
    fn ble_replies_use_device_index_ff_and_long_reports() {
        let ping = parse_read_output(BLE_PING).unwrap();
        assert_eq!(ping.len(), 20);
        assert!(is_ping_reply(&ping, 0xFF));
        let feature = parse_read_output(BLE_FEATURE).unwrap();
        assert_eq!(parse_feature_index(&feature).unwrap(), Some(8));
    }

    #[test]
    fn find_reply_skips_unrelated_input_reports() {
        let reports = parse_read_outputs(BLE_NOISE);
        assert_eq!(reports.len(), 2, "mouse movement report plus the reply");
        assert_eq!(reports[0][0], 0x02);
        let request = get_feature(0xFF, CHANGE_HOST_FEATURE_ID);
        let reply = find_reply(&reports, &request).unwrap();
        assert_eq!(parse_feature_index(reply).unwrap(), Some(8));
        // Only noise, or a reply for another device/feature: no match.
        assert!(find_reply(&reports[..1], &request).is_none());
        let other = get_feature(0x01, CHANGE_HOST_FEATURE_ID);
        assert!(find_reply(&reports, &other).is_none());
        let other_fn = ping(0xFF);
        assert!(find_reply(&reports, &other_fn).is_none());
    }

    #[test]
    fn find_reply_accepts_error_reports_for_the_device() {
        let err = vec![0x10, 0x01, 0x8F, 0x09, 0x0A, 0x09, 0x00];
        let request = get_feature(1, CHANGE_HOST_FEATURE_ID);
        assert_eq!(find_reply(std::slice::from_ref(&err), &request), Some(&err));
        let wrong_device = vec![0x10, 0x02, 0x8F, 0x09, 0x0A, 0x09, 0x00];
        assert!(find_reply(&[wrong_device], &request).is_none());
    }

    #[test]
    fn multiple_blocks_skip_timeouts() {
        let out = "read 0 bytes:\n 00 00\nread 2 bytes:\n 0A 0B\nread 0 bytes:\n 00 00";
        assert_eq!(parse_read_outputs(out), vec![vec![0x0A, 0x0B]]);
    }

    #[test]
    fn parse_reply_rejects_non_reports() {
        assert!(parse_reply(&[]).is_err());
        assert!(parse_reply(&[0x20, 1, 0, 0x1A, 0, 0, 0]).is_err());
        assert!(parse_reply(&[0x11, 1, 0, 0x1A]).is_err());
        let hidpp2_err = [0x11, 0x01, 0xFF, 0x09, 0x1A, 0x02, 0x00];
        assert_eq!(
            parse_reply(&hidpp2_err),
            Err(HostswitchError::ToolFailed("HID++ error code 2".into()))
        );
    }

    #[test]
    fn feature_index_zero_means_unsupported() {
        assert_eq!(
            parse_feature_index(&[0x11, 1, 0, 0x0A, 0, 0, 0]).unwrap(),
            None
        );
    }

    #[test]
    fn host_info_layout_is_zero_based_slot() {
        let reply = [0x11, 0x01, 0x09, 0x0A, 3, 1, 0];
        let info = parse_host_info(&reply).unwrap();
        assert_eq!(info.host_count, 3);
        assert_eq!(info.current_slot, Slot::new(2));
        assert_eq!(
            parse_host_info(&[0x11, 1, 9, 0x0A, 3, 7, 0])
                .unwrap()
                .current_slot,
            None,
            "out of range"
        );
        assert_eq!(
            parse_host_info(&[0x11, 1, 9, 0x0A, 3, 255, 0])
                .unwrap()
                .current_slot,
            None,
            "no overflow"
        );
    }
}
