// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use anyhow::Result;
use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_unix_ts() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_nanos()
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
