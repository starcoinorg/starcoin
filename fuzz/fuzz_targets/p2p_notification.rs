//! Payloads a peer controls on the p2p layer: the handshake status, the generic data and chain
//! status updates, and the notification protocols, each followed by the stateless work the
//! node does on receipt before handing the message to a service.
#![no_main]

use bcs_ext::BCSCodec;
use libfuzzer_sys::fuzz_target;
use network_api::messages::{
    NotificationMessage, ANNOUNCEMENT_PROTOCOL_NAME, BLOCK_PROTOCOL_NAME, TXN_PROTOCOL_NAME,
};
use network_p2p::business_layer_handle::BusinessLayerHandle;
use network_p2p::PeerId;
use starcoin_network::network_p2p_handle::Networkp2pHandle;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};

// The local node's chain is `ChainInfo::default()`, which the seed handshake also uses, so
// mutations get past the genesis check.
thread_local! {
    static PEER: PeerId = PeerId::random();
}

fn handle_notification(protocol: &str, bytes: &[u8]) {
    let Ok(message) = NotificationMessage::decode_notification(protocol, bytes) else {
        return;
    };
    // Mirrors NetworkActorService::handle_network_message.
    match &message {
        NotificationMessage::Transactions(msg) => {
            for txn in &msg.txns {
                txn.id();
            }
        }
        NotificationMessage::CompactBlock(msg) => {
            let header = msg.compact_block.header.clone();
            header.id();
            ChainStatus::new(header, msg.block_info.clone());
        }
        NotificationMessage::Announcement(msg) => {
            msg.ids.len();
        }
    }
    message
        .encode_notification()
        .expect("a decoded notification must encode again for relay");
}

fuzz_target!(|data: &[u8]| {
    let Some((selector, bytes)) = data.split_first() else {
        return;
    };
    match selector % 6 {
        0 => handle_notification(TXN_PROTOCOL_NAME, bytes),
        1 => handle_notification(BLOCK_PROTOCOL_NAME, bytes),
        2 => handle_notification(ANNOUNCEMENT_PROTOCOL_NAME, bytes),
        3 => {
            let handle = Networkp2pHandle::new(ChainInfo::default());
            let _ = handle.handshake(PEER.with(|p| *p), bytes.to_vec());
        }
        4 => {
            let mut handle = Networkp2pHandle::new(ChainInfo::default());
            if handle.update_generic_data(bytes).is_ok() {
                handle
                    .get_generic_data()
                    .expect("updated status must encode");
            }
        }
        _ => {
            let mut handle = Networkp2pHandle::new(ChainInfo::default());
            if handle.update_status(bytes).is_ok() {
                handle
                    .get_generic_data()
                    .expect("updated status must encode");
            }
            if let Ok(status) = ChainStatus::decode(bytes) {
                status.head().id();
            }
        }
    }
});
