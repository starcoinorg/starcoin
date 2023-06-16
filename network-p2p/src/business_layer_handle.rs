use std::borrow::Cow;

use anyhow::Error;
use libp2p::PeerId;
use sc_peerset::{ReputationChange, SetId};

use crate::protocol::{generic_proto::NotificationsSink, CustomMessageOutcome};

/// The above layer must implement this trait to complete some business logic related to the network.
pub trait BusinessLayerHandle {
    /// To verify whether the connection is qualified.
    /// if handshaking is successful, return CustomMessageOutcome::NotificationStreamOpened
    /// otherwise, return Error
    fn handshake(
        &self,
        peer_id: PeerId,
        set_id: SetId,
        protocol_name: Cow<'static, str>,
        received_handshake: Vec<u8>,
        notifications_sink: NotificationsSink,
    ) -> Result<CustomMessageOutcome, ReputationChange>;

    fn build_handshake_msg(
        &mut self,
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
    ) -> Result<Vec<u8>, Error>;

    /// Return the generic data related to the above layers
    fn get_generic_data(&self) -> Result<Vec<u8>, Error>;

    /// Update generic data
    fn update_generic_data(&mut self, peer_info: &[u8]) -> Result<(), Error>;

    /// Update business status
    fn update_status(&mut self, peer_status: &[u8]) -> Result<(), Error>;
}
