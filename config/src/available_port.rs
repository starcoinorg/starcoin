// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use parking_lot::Mutex;

static USED_PORTS: Lazy<Mutex<Vec<u16>>> = Lazy::new(|| Mutex::new(vec![]));

pub fn get_available_port_from(start_port: u16) -> u16 {
    for i in 0..100 {
        let port_to_check = start_port + i;

        if !check_port_in_use(port_to_check) {
            return port_to_check;
        }
    }
    panic!("Error: could not find an available port");
}

/// check if `port` is available.
fn check_port_in_use(port: u16) -> bool {
    if USED_PORTS.lock().contains(&port) {
        return true;
    }
    use std::net::TcpStream;
    let in_use = match TcpStream::connect(("0.0.0.0", port)) {
        Ok(_) => true,
        Err(_e) => false,
    };
    if !in_use {
        USED_PORTS.lock().push(port);
    };
    in_use
}

pub fn get_random_available_port() -> u16 {
    for _ in 0..3 {
        if let Ok(port) = get_ephemeral_port() {
            if !check_port_in_use(port) {
                return port;
            }
        }
    }
    panic!("Error: could not find an available port");
}

pub fn get_random_available_ports(num: usize) -> Vec<u16> {
    let mut ports = vec![0u16; num];

    for p in ports.iter_mut() {
        *p = get_random_available_port();
    }
    ports
}

fn get_ephemeral_port() -> ::std::io::Result<u16> {
    use std::net::{TcpListener, TcpStream};

    // Request a random available port from the OS
    let listener = TcpListener::bind(("localhost", 0))?;
    let addr = listener.local_addr()?;

    // Create and accept a connection (which we'll promptly drop) in order to force the port
    // into the TIME_WAIT state, ensuring that the port will be reserved from some limited
    // amount of time (roughly 60s on some Linux systems)
    let _sender = TcpStream::connect(addr)?;
    let _incoming = listener.accept()?;

    Ok(addr.port())
}

#[cfg(test)]
mod tests {
    use super::check_port_in_use;
    use crate::get_random_available_port;
    use std::collections::HashSet;
    use std::net::TcpListener;

    #[test]
    fn test_port_in_use() -> std::io::Result<()> {
        let port = 11110;
        let _listener1 = TcpListener::bind(("0.0.0.0", port))?;

        assert!(check_port_in_use(port));
        Ok(())
    }

    #[test]
    fn test_random_ports() {
        let mut set = HashSet::new();
        let count = 100;
        for _i in 0..count {
            set.insert(get_random_available_port());
        }
        assert_eq!(
            count,
            set.len(),
            "get_random_available_port return repeat ports."
        );
    }
}
