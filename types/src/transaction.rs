// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;

use crate::contract_event::ContractEvent;
use crate::write_set::WriteSet;
use actix::prelude::Message;
use libra_crypto::{
    ed25519::*,
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type Version = u64;

/// RawTransaction is the portion of a transaction that a client signs
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct RawTransaction {
    /// Sender's address.
    sender: AccountAddress,
    // Sequence number of this transaction corresponding to sender's account.
    sequence_number: u64,
    // The transaction script to execute.
    payload: TransactionPayload,
    // Maximal total gas specified by wallet to spend for this transaction.
    max_gas_amount: u64,
    // Maximal price can be paid per gas.
    gas_unit_price: u64,
}

impl RawTransaction {
    /// Create a new `RawTransaction` with a payload.
    ///
    /// It can be either to publish a module, to execute a script, or to issue a writeset
    /// transaction.
    pub fn new(
        sender: AccountAddress,
        sequence_number: u64,
        payload: TransactionPayload,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
        }
    }

    pub fn sender(&self) -> AccountAddress {
        self.sender
    }
}

impl CryptoHash for RawTransaction {
    type Hasher = RawTransactionHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            scs::to_bytes(self)
                .expect("Failed to serialize RawTransaction")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    Mock,
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, CryptoHasher, Message)]
#[rtype(result = "()")]
pub struct SignedTransaction {
    /// The raw transaction
    raw_txn: RawTransaction,

    /// Sender's public key. When checking the signature, we first need to check whether this key
    /// is indeed the pre-image of the pubkey hash stored under sender's account.
    public_key: Ed25519PublicKey,

    /// Signature of the transaction that correspond to the public key
    signature: Ed25519Signature,
}

impl SignedTransaction {
    pub fn new(
        raw_txn: RawTransaction,
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    ) -> SignedTransaction {
        SignedTransaction {
            raw_txn,
            public_key,
            signature,
        }
    }

    pub fn raw_txn(&self) -> &RawTransaction {
        &self.raw_txn
    }

    pub fn public_key(&self) -> Ed25519PublicKey {
        self.public_key.clone()
    }

    pub fn signature(&self) -> Ed25519Signature {
        self.signature.clone()
    }

    pub fn sender(&self) -> AccountAddress {
        self.raw_txn.sender
    }

    pub fn into_raw_transaction(self) -> RawTransaction {
        self.raw_txn
    }

    pub fn sequence_number(&self) -> u64 {
        self.raw_txn.sequence_number
    }

    pub fn payload(&self) -> &TransactionPayload {
        &self.raw_txn.payload
    }

    pub fn max_gas_amount(&self) -> u64 {
        self.raw_txn.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> u64 {
        self.raw_txn.gas_unit_price
    }
}

impl fmt::Debug for SignedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SignedTransaction {{ \n \
             {{ raw_txn: {:#?}, \n \
             public_key: {:#?}, \n \
             signature: {:#?}, \n \
             }} \n \
             }}",
            self.raw_txn, self.public_key, self.signature,
        )
    }
}

//TODO define code.
type StatusCode = u64;

pub struct TransactionInfo {
    /// The hash of this transaction.
    pub transaction_hash: HashValue,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: HashValue,

    /// The amount of gas used.
    gas_used: u64,

    /// The major status. This will provide the general error class. Note that this is not
    /// particularly high fidelity in the presence of sub statuses but, the major status does
    /// determine whether or not the transaction is applied to the global state or not.
    major_status: StatusCode,
}

impl TransactionInfo {
    /// Constructs a new `TransactionInfo` object using transaction hash, state root hash and event
    /// root hash.
    pub fn new(
        transaction_hash: HashValue,
        state_root_hash: HashValue,
        event_root_hash: HashValue,
        gas_used: u64,
        major_status: StatusCode,
    ) -> TransactionInfo {
        TransactionInfo {
            transaction_hash,
            state_root_hash,
            event_root_hash,
            gas_used,
            major_status,
        }
    }

    /// Returns the hash of this transaction.
    pub fn transaction_hash(&self) -> HashValue {
        self.transaction_hash
    }

    /// Returns root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub fn state_root_hash(&self) -> HashValue {
        self.state_root_hash
    }

    /// Returns the root hash of Merkle Accumulator storing all events emitted during this
    /// transaction.
    pub fn event_root_hash(&self) -> HashValue {
        self.event_root_hash
    }

    /// Returns the amount of gas used by this transaction.
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn major_status(&self) -> StatusCode {
        self.major_status
    }
}

//TODO define TransactionStatus
type TransactionStatus = u64;

/// The output of executing a transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOutput {
    /// The list of writes this transaction intends to do.
    write_set: WriteSet,

    /// The list of events emitted during this transaction.
    events: Vec<ContractEvent>,

    /// The amount of gas used during execution.
    gas_used: u64,

    /// The execution status.
    status: TransactionStatus,
}

impl TransactionOutput {
    pub fn new(
        write_set: WriteSet,
        events: Vec<ContractEvent>,
        gas_used: u64,
        status: TransactionStatus,
    ) -> Self {
        TransactionOutput {
            write_set,
            events,
            gas_used,
            status,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }
}
