// SPDX-License-Identifier: Apache-2.0

//! Pure parser for `hidapitester --list-detail` (fixture: hidapitester-list-detail-receiver.txt).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub vendor_id: u16,
    pub product_id: u16,
    pub description: String,
    pub usage_page: u16,
    pub usage: u16,
    pub interface: i32,
    pub bus_type: String,
    pub path: String,
}

impl Endpoint {
    pub fn is_bluetooth(&self) -> bool {
        self.bus_type.starts_with("Bluetooth")
    }
}

pub fn parse(output: &str) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();
    let mut current: Option<Endpoint> = None;

    for line in output.lines() {
        if !line.starts_with(' ') {
            endpoints.extend(current.take());
            current = parse_header(line);
            continue;
        }
        let (Some(endpoint), Some((key, value))) = (current.as_mut(), line.split_once(':')) else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "usagePage" => endpoint.usage_page = hex(value),
            "usage" => endpoint.usage = hex(value),
            "interface" => endpoint.interface = value.parse().unwrap_or(-1),
            "bus_type" => endpoint.bus_type = value.to_string(),
            "path" => endpoint.path = value.to_string(),
            _ => {}
        }
    }
    endpoints.extend(current);
    endpoints
}

/// "046D/C52B: Logitech - USB Receiver"
fn parse_header(line: &str) -> Option<Endpoint> {
    let (ids, description) = line.split_once(':')?;
    let (vendor, product) = ids.split_once('/')?;
    Some(Endpoint {
        vendor_id: u16::from_str_radix(vendor.trim(), 16).ok()?,
        product_id: u16::from_str_radix(product.trim(), 16).ok()?,
        description: description.trim().to_string(),
        usage_page: 0,
        usage: 0,
        interface: -1,
        bus_type: String::new(),
        path: String::new(),
    })
}

fn hex(s: &str) -> u16 {
    u16::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOGITECH: &str = include_str!("../tests/fixtures/hidapitester-list-detail-receiver.txt");
    // Contains non-UTF8 device names (decoded lossily).
    const ALL: &[u8] = include_bytes!("../tests/fixtures/hidapitester-list-all.txt");

    #[test]
    fn parses_the_receiver_collections() {
        let endpoints = parse(LOGITECH);
        assert!(endpoints.len() >= 4);
        assert!(
            endpoints
                .iter()
                .all(|e| e.vendor_id == 0x046D && e.product_id == 0xC52B)
        );
        let vendor_short = endpoints
            .iter()
            .find(|e| e.usage_page == 0xFF00 && e.usage == 1)
            .unwrap();
        assert_eq!(vendor_short.interface, 2);
        assert!(vendor_short.bus_type.starts_with("USB"));
        assert!(vendor_short.description.contains("USB Receiver"));
    }

    #[test]
    fn tolerates_garbage_and_other_vendors() {
        assert!(parse("").is_empty());
        assert!(parse("complete nonsense\n  key: value\n").is_empty());
        // The mixed-vendor listing must parse without panicking and keep every vendor.
        let all = parse(&String::from_utf8_lossy(ALL));
        assert!(all.iter().any(|e| e.vendor_id == 0x046D));
    }
}
