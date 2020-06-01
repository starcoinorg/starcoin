// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

//! Network event types. These are are not the part of the protocol, but rather
//! events that happen on the network like DHT get/put results received.

use bytes::Bytes;
use libp2p::core::PeerId;
use libp2p::kad::record::Key;
use std::borrow::Cow;
use types::peer_info::PeerInfo;

/// Events generated by DHT as a response to get_value and put_value requests.
#[derive(Debug, Clone)]
#[must_use]
pub enum DhtEvent {
    /// The value was found.
    ValueFound(Vec<(Key, Vec<u8>)>),

    /// The requested record has not been found in the DHT.
    ValueNotFound(Key),

    /// The record has been successfully inserted into the DHT.
    ValuePut(Key),

    /// An error has occurred while putting a record into the DHT.
    ValuePutFailed(Key),
}

/// Type for events generated by networking layer.
#[derive(Debug, Clone)]
#[must_use]
pub enum Event {
    /// Event generated by a DHT.
    Dht(DhtEvent),

    /// Opened a substream with the given node with the given notifications protocol.
    ///
    /// The protocol is always one of the notification protocols that have been registered.
    NotificationStreamOpened {
        /// Node we opened the substream with.
        remote: PeerId,
        info: Box<PeerInfo>,
    },

    /// Closed a substream with the given node. Always matches a corresponding previous
    /// `NotificationStreamOpened` message.
    NotificationStreamClosed {
        /// Node we closed the substream with.
        remote: PeerId,
    },

    /// Received one or more messages from the given node using the given protocol.
    NotificationsReceived {
        /// Node we received the message from.
        remote: PeerId,
        protocol_name: Cow<'static, [u8]>,
        /// Concerned protocol and associated message.
        messages: Vec<Bytes>,
    },
}
