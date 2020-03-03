pub mod event;
pub mod generic_proto;
pub mod util;

use bytes::{Bytes, BytesMut};
use libp2p::PeerId;

#[derive(Debug)]
pub enum CustomMessageOutcome {
    NotificationStreamOpened {
        remote: PeerId,
    },
    /// Notification protocols have been closed with a remote.
    NotificationStreamClosed {
        remote: PeerId,
    },
    /// Messages have been received on one or more notifications protocols.
    NotificationsReceived {
        remote: PeerId,
        messages: Vec<Bytes>,
    },
    None,
}
