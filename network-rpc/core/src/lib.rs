// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod delegates;
pub mod server;

use futures::future::BoxFuture;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod prelude {
    pub use crate::NetRpcError;
    pub use crate::PeerId;
    pub use network_rpc_derive::net_rpc;
}

pub mod export {
    pub mod log {
        pub use log::{debug, error, info, warn};
    }

    pub mod scs {
        pub use scs::{from_bytes, SCSCodec};
    }
}

//TODO find a suitable place for this type.
use crate::server::NetworkRpcServer;
use futures::FutureExt;
pub use starcoin_types::peer_info::PeerId;
use std::convert::TryFrom;

#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(
    Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive,
)]
pub enum RpcErrorCode {
    BadRequest = 400,
    Forbidden = 403,
    MethodNotFound = 404,
    InternalError = 500,
    ServerUnavailable = 503,
    Unknown = 1000,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetRpcError {
    error_code: u32,
    /// Message
    message: String,
}

impl NetRpcError {
    pub fn new(error_code: RpcErrorCode, msg: String) -> Self {
        Self {
            error_code: error_code.into(),
            message: msg,
        }
    }

    pub fn client_err<M>(msg: M) -> Self
    where
        M: ToString,
    {
        Self::new(RpcErrorCode::BadRequest, msg.to_string())
    }

    pub fn forbidden(reason: &str) -> Self {
        Self::new(
            RpcErrorCode::Forbidden,
            format!("Request forbidden : {}", reason),
        )
    }

    pub fn method_not_fount(rpc_path: String) -> Self {
        Self::new(
            RpcErrorCode::MethodNotFound,
            format!("Request method {} not found", rpc_path),
        )
    }

    pub fn error_code(&self) -> RpcErrorCode {
        RpcErrorCode::try_from(self.error_code).unwrap_or(RpcErrorCode::Unknown)
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl std::error::Error for NetRpcError {}

impl std::fmt::Display for NetRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Network Rpc error: {}", self.message)
    }
}

pub type Result<T, E = NetRpcError> = core::result::Result<T, E>;

impl From<anyhow::Error> for NetRpcError {
    fn from(any_err: anyhow::Error) -> Self {
        match any_err.downcast::<NetRpcError>() {
            Ok(rpc_err) => rpc_err,
            Err(any_err) => {
                //TODO do more convert.
                NetRpcError::new(RpcErrorCode::InternalError, any_err.to_string())
            }
        }
    }
}

pub trait RawRpcServer {
    /// peer_id: the client PeerID who send request.
    /// RawRpc server should convert all error to NetRpcError
    /// And the transport serialize Result<Vec<u8>,NetRpcError> to bytes.
    fn handle_raw_request(
        &self,
        peer_id: PeerId,
        rpc_path: String,
        message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>>;
}

pub trait RawRpcClient {
    /// peer_id: the target PeerID send request to, if peer_id is absent, auto select a peer_id.
    /// RawRpcClient's result Vec<u8> is Result<Vec<u8>, NetRpcError>'s bytes.
    fn send_raw_request(
        &self,
        peer_id: Option<PeerId>,
        rpc_path: String,
        message: Vec<u8>,
        timeout: Duration,
    ) -> BoxFuture<anyhow::Result<Vec<u8>>>;
}

/// A in memory rpc client witch hold a server, just for test
pub struct InmemoryRpcClient {
    self_peer_id: PeerId,
    server: NetworkRpcServer,
}

impl InmemoryRpcClient {
    pub fn new(self_peer_id: PeerId, server: NetworkRpcServer) -> Self {
        Self {
            self_peer_id,
            server,
        }
    }
}

impl RawRpcClient for InmemoryRpcClient {
    fn send_raw_request(
        &self,
        _peer_id: Option<PeerId>,
        rpc_path: String,
        message: Vec<u8>,
        _timeout: Duration,
    ) -> BoxFuture<anyhow::Result<Vec<u8>>> {
        Box::pin(
            self.server
                .handle_raw_request(self.self_peer_id.clone(), rpc_path, message)
                .then(|result| async move { anyhow::Result::Ok(scs::to_bytes(&result).unwrap()) }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_serialize() {
        let str_result: Result<String, NetRpcError> = Result::Ok("test".to_string());
        let bytes = scs::to_bytes(&str_result).unwrap();
        println!("bytes:{:?}", bytes);
        let str_result2: Result<String, NetRpcError> = scs::from_bytes(bytes.as_slice()).unwrap();
        println!("result:{:?}", str_result2);
        assert_eq!(str_result, str_result2);
    }
}
