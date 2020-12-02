// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Substrate network possible errors.

use crate::config::TransportConfig;
use libp2p::core::{Multiaddr, PeerId};
use starcoin_metrics::PrometheusError;
use std::borrow::Cow;
use std::fmt;

/// Result type alias for the network.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the network.
#[derive(derive_more::Display, derive_more::From)]
pub enum Error {
    /// Io error
    Io(std::io::Error),
    // Client error
    // Client(std::io::Error),
    /// The same bootnode (based on address) is registered with two different peer ids.
    #[display(
        fmt = "The same bootnode (`{}`) is registered with two different peer ids: `{}` and `{}`",
        address,
        first_id,
        second_id
    )]
    DuplicateBootnode {
        /// The address of the bootnode.
        address: Multiaddr,
        /// The first peer id that was found for the bootnode.
        first_id: PeerId,
        /// The second peer id that was found for the bootnode.
        second_id: PeerId,
    },

    /// Prometheus metrics error.
    Prometheus(PrometheusError),
    /// The network addresses are invalid because they don't match the transport.
    #[display(
        fmt = "The following addresses are invalid because they don't match the transport: {:?}",
        addresses
    )]
    AddressesForAnotherTransport {
        /// Transport used.
        transport: TransportConfig,
        /// The invalid addresses.
        addresses: Vec<Multiaddr>,
    },
    /// The same request-response protocol has been registered multiple times.
    #[display(
        fmt = "Request-response protocol registered multiple times: {}",
        protocol
    )]
    DuplicateRequestResponseProtocol {
        /// Name of the protocol registered multiple times.
        protocol: Cow<'static, str>,
    },
}

// Make `Debug` use the `Display` implementation.
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(ref err) => Some(err),
            Error::DuplicateBootnode { .. } => None,
            Error::Prometheus(ref err) => Some(err),
            Error::AddressesForAnotherTransport { .. } => None,
            Error::DuplicateRequestResponseProtocol { .. } => None,
        }
    }
}
