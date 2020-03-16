// Copyright 2015-2019 Parity Technologies (UK) Ltd.

use std;
use std::error::Error as StdError;
use ethereum_types::H256;
use jsonrpc_tcp_server::PushMessageError;

#[derive(Debug, Clone)]
pub enum Error {
	NoWork,
	NoWorkers,
	Io(String),
	Tcp(String),
	Dispatch(String),
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::Io(err.description().to_owned())
	}
}

impl From<PushMessageError> for Error {
	fn from(err: PushMessageError) -> Self {
		Error::Tcp(format!("Push message error: {:?}", err))
	}
}

/// Interface that can provide pow/blockchain-specific responses for the clients
pub trait JobDispatcher: Send + Sync {
	// json for initial client handshake
	fn initial(&self) -> Option<String> { None }
	// json for difficulty dispatch
	fn difficulty(&self) -> Option<String> { None }
	// json for job update given worker_id (payload manager should split job!)
	fn job(&self) -> Option<String> { None }
	// miner job result
	fn submit(&self, payload: Vec<String>) -> Result<(), Error>;
}

/// Interface that can handle requests to push job for workers
pub trait PushWorkHandler: Send + Sync {
	/// push the same work package for all workers (`payload`: json of pow-specific set of work specification)
	fn push_work_all(&self, payload: String) -> Result<(), Error>;

	/// push the work packages worker-wise (`payload`: json of pow-specific set of work specification)
	fn push_work(&self, payloads: Vec<String>) -> Result<(), Error>;
}

pub struct ServiceConfiguration {
	pub io_path: String,
	pub listen_addr: String,
	pub port: u16,
	pub secret: Option<H256>,
}
