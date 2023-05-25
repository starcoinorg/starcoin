use anyhow::Error;

/// The above layer must implement this trait to complete some business logic related to the network.
pub trait BusinessLayerHandle {
    /// To verify whether the connection is qualified.
    fn handshake(&self, peer_info: &[u8]) -> Result<(), (&'static str, String)>;

    /// Return the generic data related to the above layers
    fn get_generic_data(&self) -> Result<Vec<u8>, Error>;

    /// Update generic data
    fn update_generic_data(&mut self, peer_info: &[u8]) -> Result<(), Error>;

    /// Update business status
    fn update_status(&mut self, peer_status: &[u8]) -> Result<(), Error>;
}
