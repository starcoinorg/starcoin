// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use anyhow::Result;
use std::net::Ipv4Addr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_unix_ts() -> u128 {
    get_unix_duration().as_nanos()
}

pub fn get_unix_ts_as_millis() -> u128 {
    get_unix_duration().as_millis()
}

fn get_unix_duration() -> Duration {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}

pub fn is_global(ip: Ipv4Addr) -> bool {
    // check if this address is 192.0.0.9 or 192.0.0.10. These addresses are the only two
    // globally routable addresses in the 192.0.0.0/24 range.
    if u32::from(ip) == 0xc000_0009 || u32::from(ip) == 0xc000_000a {
        return true;
    }
    !ip.is_private()
        && !ip.is_loopback()
        && !ip.is_link_local()
        && !ip.is_broadcast()
        && !ip.is_documentation()
        && !is_shared(ip)
        && !is_ietf_protocol_assignment(ip)
        && !is_reserved(ip)
        && !is_benchmarking(ip)
        // Make sure the address is not in 0.0.0.0/8
        && ip.octets()[0] != 0
}

fn is_shared(ip: Ipv4Addr) -> bool {
    ip.octets()[0] == 100 && (ip.octets()[1] & 0b1100_0000 == 0b0100_0000)
}

fn is_ietf_protocol_assignment(ip: Ipv4Addr) -> bool {
    ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 0
}

fn is_reserved(ip: Ipv4Addr) -> bool {
    ip.octets()[0] & 240 == 240 && !ip.is_broadcast()
}

fn is_benchmarking(ip: Ipv4Addr) -> bool {
    ip.octets()[0] == 198 && (ip.octets()[1] & 0xfe) == 18
}

#[cfg(test)]
mod tests {
    use crate::helper::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_192_0_0() {
        let ip_9 = Ipv4Addr::new(192, 0, 0, 9);
        assert!(is_global(ip_9));

        let ip_10 = Ipv4Addr::new(192, 0, 0, 10);
        assert!(is_global(ip_10));
    }

    #[test]
    fn test_is_shared() {
        let ip = Ipv4Addr::new(100, 64, 0, 1);
        assert!(is_shared(ip));
    }

    #[test]
    fn test_is_ietf_protocol_assignment() {
        let ip = Ipv4Addr::new(192, 0, 0, 1);
        assert!(is_ietf_protocol_assignment(ip));
    }

    #[test]
    fn test_is_reserved() {
        let ip_1 = Ipv4Addr::new(241, 1, 1, 100);
        assert!(is_reserved(ip_1));

        let ip_2 = Ipv4Addr::new(255, 255, 255, 255);
        assert!(!is_reserved(ip_2));
    }

    #[test]
    fn test_is_benchmarking() {
        let ip = Ipv4Addr::new(198, 18, 0, 10);
        assert!(is_benchmarking(ip));
    }
}
