// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::{genesis_address, STC_TOKEN_CODE_STR};
use crate::block_metadata::BlockMetadata;
use crate::genesis_config::ChainId;
use crate::transaction::authenticator::{AccountPublicKey, TransactionAuthenticator};
use crate::{
    account_address::AccountAddress,
    contract_event::ContractEvent,
    vm_status::{DiscardedVMStatus, KeptVMStatus},
    vm_status::{StatusCode, VMStatus},
    write_set::WriteSet,
};
use anyhow::{format_err, Error, Result};
use bcs_ext::Sample;
use serde::{Deserialize, Deserializer, Serialize};
use starcoin_accumulator::inmemory::InMemoryAccumulator;
use starcoin_crypto::multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature};
use starcoin_crypto::{
    ed25519::*,
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    traits::*,
    HashValue,
};
use std::ops::Deref;
use std::{convert::TryFrom, fmt};

pub use error::CallError;
pub use error::Error as TransactionError;
pub use module::Module;
pub use package::Package;
pub use pending_transaction::{Condition, PendingTransaction};
use schemars::{self, JsonSchema};
pub use script::{
    ArgumentABI, Script, ScriptABI, ScriptFunction, ScriptFunctionABI, TransactionScriptABI,
    TypeArgumentABI,
};
use starcoin_crypto::hash::SPARSE_MERKLE_PLACEHOLDER_HASH;
use std::str::FromStr;
pub use transaction_argument::{
    parse_transaction_argument, parse_transaction_arguments, TransactionArgument,
};

pub mod authenticator;
mod error;
pub mod helpers;
mod module;
mod package;
mod pending_transaction;
mod script;
#[cfg(test)]
mod tests;
mod transaction_argument;

pub type Version = u64; // Height - also used for MVCC in StateDB

/// RawUserTransaction is the portion of a transaction that a client signs
#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
pub struct RawUserTransaction {
    /// Sender's address.
    #[schemars(with = "String")]
    sender: AccountAddress,
    // Sequence number of this transaction corresponding to sender's account.
    sequence_number: u64,
    // The transaction script to execute.
    payload: TransactionPayload,

    // Maximal total gas specified by wallet to spend for this transaction.
    max_gas_amount: u64,
    // Maximal price can be paid per gas.
    gas_unit_price: u64,
    // The token code for pay transaction gas, Default is STC token code.
    gas_token_code: String,
    // Expiration timestamp for this transaction. timestamp is represented
    // as u64 in seconds from Unix Epoch. If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
}

impl RawUserTransaction {
    /// Create a new `RawUserTransaction` with a payload.
    ///
    /// It can be either to publish a module, to execute a script, or to issue a writeset
    /// transaction.
    pub fn new(
        sender: AccountAddress,
        sequence_number: u64,
        payload: TransactionPayload,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
        gas_token_code: String,
    ) -> Self {
        RawUserTransaction {
            sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            gas_token_code,
            expiration_timestamp_secs,
            chain_id,
        }
    }

    pub fn new_with_default_gas_token(
        sender: AccountAddress,
        sequence_number: u64,
        payload: TransactionPayload,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        RawUserTransaction {
            sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            gas_token_code: STC_TOKEN_CODE_STR.to_string(),
            expiration_timestamp_secs,
            chain_id,
        }
    }

    /// Create a new `RawUserTransaction` with a script.
    ///
    /// A script transaction contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script(
        sender: AccountAddress,
        sequence_number: u64,
        script: Script,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        RawUserTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::Script(script),
            max_gas_amount,
            gas_unit_price,
            gas_token_code: STC_TOKEN_CODE_STR.to_string(),
            expiration_timestamp_secs,
            chain_id,
        }
    }

    /// Create a new `RawTransaction` with a script function.
    ///
    /// A script transaction contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script_function(
        sender: AccountAddress,
        sequence_number: u64,
        script_function: ScriptFunction,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        RawUserTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::ScriptFunction(script_function),
            max_gas_amount,
            gas_unit_price,
            gas_token_code: STC_TOKEN_CODE_STR.to_string(),
            expiration_timestamp_secs,
            chain_id,
        }
    }

    /// Create a new `RawUserTransaction` with a package to publish.
    pub fn new_package(
        sender: AccountAddress,
        sequence_number: u64,
        package: Package,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        RawUserTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::Package(package),
            max_gas_amount,
            gas_unit_price,
            gas_token_code: STC_TOKEN_CODE_STR.to_string(),
            expiration_timestamp_secs,
            chain_id,
        }
    }

    /// Create a new `RawUserTransaction` with a module to publish.
    ///
    /// A module transaction is the only way to publish code. Only one module per transaction
    /// can be published.
    pub fn new_module(
        sender: AccountAddress,
        sequence_number: u64,
        module: Module,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        Self::new_package(
            sender,
            sequence_number,
            Package::new_with_module(module).expect("build package with module should success."),
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        )
    }

    /// Signs the given `RawUserTransaction`. Note that this consumes the `RawUserTransaction` and turns it
    /// into a `SignatureCheckedTransaction`.
    ///
    /// For a transaction that has just been signed, its signature is expected to be valid.
    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> Result<SignatureCheckedTransaction> {
        let signature = private_key.sign(&self);
        Ok(SignatureCheckedTransaction(SignedUserTransaction::ed25519(
            self, public_key, signature,
        )))
    }

    pub fn into_payload(self) -> TransactionPayload {
        self.payload
    }

    /// Return the sender of this transaction.
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }
    pub fn max_gas_amount(&self) -> u64 {
        self.max_gas_amount
    }
    pub fn gas_unit_price(&self) -> u64 {
        self.gas_unit_price
    }
    pub fn gas_token_code(&self) -> String {
        self.gas_token_code.clone()
    }
    pub fn expiration_timestamp_secs(&self) -> u64 {
        self.expiration_timestamp_secs
    }
    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }
    pub fn payload(&self) -> &TransactionPayload {
        &self.payload
    }

    pub fn txn_size(&self) -> usize {
        bcs_ext::to_bytes(self)
            .expect("Unable to serialize RawUserTransaction")
            .len()
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self> {
        Self::from_bytes(hex::decode(hex)?)
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        bcs_ext::from_bytes(bytes.as_ref())
    }

    pub fn to_hex(&self) -> String {
        format!(
            "0x{}",
            hex::encode(
                bcs_ext::to_bytes(&self).expect("Serialize RawUserTransaction should success.")
            )
        )
    }

    pub fn mock() -> Self {
        Self::mock_by_sender(AccountAddress::random())
    }

    pub fn mock_by_sender(sender: AccountAddress) -> Self {
        Self::new_with_default_gas_token(
            sender,
            0,
            TransactionPayload::Script(Script::new(vec![], vec![], vec![])),
            0,
            0,
            u64::max_value(),
            ChainId::test(),
        )
    }

    pub fn mock_from(compiled_script: Vec<u8>) -> Self {
        Self::new_with_default_gas_token(
            AccountAddress::ZERO,
            0,
            TransactionPayload::Script(Script::new(compiled_script, vec![], vec![])),
            600,
            0,
            u64::max_value(),
            ChainId::test(),
        )
    }
}

impl FromStr for RawUserTransaction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        Self::from_hex(s)
    }
}

impl TryFrom<&[u8]> for RawUserTransaction {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<Vec<u8>> for RawUserTransaction {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

impl Sample for RawUserTransaction {
    fn sample() -> Self {
        Self::new_module(
            genesis_address(),
            0,
            Module::sample(),
            0,
            1,
            3600,
            ChainId::test(),
        )
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TransactionPayload {
    /// A transaction that executes code.
    Script(Script),
    /// A transaction that publish or update module code by a package.
    Package(Package),
    /// A transaction that executes an existing script function published on-chain.
    ScriptFunction(ScriptFunction),
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum TransactionPayloadType {
    Script = 0,
    Package = 1,
    ScriptFunction = 2,
}

impl TransactionPayload {
    pub fn payload_type(&self) -> TransactionPayloadType {
        match self {
            TransactionPayload::Script(_) => TransactionPayloadType::Script,
            TransactionPayload::Package(_) => TransactionPayloadType::Package,
            TransactionPayload::ScriptFunction(_) => TransactionPayloadType::ScriptFunction,
        }
    }
}

impl TryFrom<u8> for TransactionPayloadType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TransactionPayloadType::Script),
            1 => Ok(TransactionPayloadType::Package),
            _ => Err(format_err!("invalid PayloadType")),
        }
    }
}

impl From<TransactionPayloadType> for u8 {
    fn from(t: TransactionPayloadType) -> Self {
        t as u8
    }
}

/// A transaction that has been signed.
///
/// A `SignedUserTransaction` is a single transaction that can be atomically executed. Clients submit
/// these to validator nodes, and the validator and executor submits these to the VM.
///
/// **IMPORTANT:** The signature of a `SignedUserTransaction` is not guaranteed to be verified. For a
/// transaction whose signature is statically guaranteed to be verified, see
/// [`SignatureCheckedTransaction`].
#[derive(Clone, Eq, PartialEq, Hash, Serialize, CryptoHasher, CryptoHash, JsonSchema)]
pub struct SignedUserTransaction {
    #[serde(skip)]
    #[schemars(skip)]
    id: Option<HashValue>,

    /// The raw transaction
    raw_txn: RawUserTransaction,

    /// Public key and signature to authenticate
    authenticator: TransactionAuthenticator,
}

#[derive(Clone, Eq, PartialEq)]
pub struct DryRunTransaction {
    /// The raw transaction
    pub raw_txn: RawUserTransaction,
    pub public_key: AccountPublicKey,
}

/// A transaction for which the signature has been verified. Created by
/// [`SignedUserTransaction::check_signature`] and [`RawUserTransaction::sign`].
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SignatureCheckedTransaction(SignedUserTransaction);

impl SignatureCheckedTransaction {
    /// Returns the `SignedUserTransaction` within.
    pub fn into_inner(self) -> SignedUserTransaction {
        self.0
    }

    /// Returns the `RawUserTransaction` within.
    pub fn into_raw_transaction(self) -> RawUserTransaction {
        self.0.into_raw_transaction()
    }
}

impl Deref for SignatureCheckedTransaction {
    type Target = SignedUserTransaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for SignedUserTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SignedTransaction {{ \n \
             {{ raw_txn: {:#?}, \n \
             authenticator: {:#?}, \n \
             }} \n \
             }}",
            self.raw_txn, self.authenticator
        )
    }
}

impl SignedUserTransaction {
    pub fn new(
        raw_txn: RawUserTransaction,
        authenticator: TransactionAuthenticator,
    ) -> SignedUserTransaction {
        let mut txn = Self {
            id: None,
            raw_txn,
            authenticator,
        };
        txn.id = Some(txn.crypto_hash());
        txn
    }

    pub fn ed25519(
        raw_txn: RawUserTransaction,
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    ) -> SignedUserTransaction {
        let authenticator = TransactionAuthenticator::ed25519(public_key, signature);
        Self::new(raw_txn, authenticator)
    }

    pub fn multi_ed25519(
        raw_txn: RawUserTransaction,
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    ) -> SignedUserTransaction {
        let authenticator = TransactionAuthenticator::multi_ed25519(public_key, signature);
        Self::new(raw_txn, authenticator)
    }

    pub fn authenticator(&self) -> TransactionAuthenticator {
        self.authenticator.clone()
    }

    pub fn raw_txn(&self) -> &RawUserTransaction {
        &self.raw_txn
    }

    pub fn sender(&self) -> AccountAddress {
        self.raw_txn.sender
    }

    pub fn into_raw_transaction(self) -> RawUserTransaction {
        self.raw_txn
    }

    pub fn sequence_number(&self) -> u64 {
        self.raw_txn.sequence_number
    }

    pub fn chain_id(&self) -> ChainId {
        self.raw_txn.chain_id
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

    pub fn gas_token_code(&self) -> &str {
        self.raw_txn.gas_token_code.as_str()
    }

    pub fn expiration_timestamp_secs(&self) -> u64 {
        self.raw_txn.expiration_timestamp_secs
    }

    pub fn raw_txn_bytes_len(&self) -> usize {
        self.raw_txn.txn_size()
    }

    /// Checks that the signature of given transaction. Returns `Ok(SignatureCheckedTransaction)` if
    /// the signature is valid.
    pub fn check_signature(self) -> Result<SignatureCheckedTransaction> {
        self.authenticator.verify(&self.raw_txn)?;
        Ok(SignatureCheckedTransaction(self))
    }

    ///TODO cfg test
    pub fn mock() -> Self {
        let (private_key, public_key) = genesis_key_pair();
        let raw_txn = RawUserTransaction::mock();
        raw_txn.sign(&private_key, public_key).unwrap().into_inner()
    }

    pub fn id(&self) -> HashValue {
        self.id
            .expect("SignedUserTransaction's id should bean Some after init.")
    }
}

impl<'de> Deserialize<'de> for SignedUserTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "SignedUserTransaction")]
        struct SignedUserTransactionData {
            raw_txn: RawUserTransaction,
            authenticator: TransactionAuthenticator,
        }
        let data = SignedUserTransactionData::deserialize(deserializer)?;
        Ok(Self::new(data.raw_txn, data.authenticator))
    }
}

impl Sample for SignedUserTransaction {
    fn sample() -> Self {
        let raw_txn = RawUserTransaction::sample();
        let (private_key, public_key) = genesis_key_pair();
        let signature = private_key.sign(&raw_txn);
        Self::ed25519(raw_txn, public_key, signature)
    }
}

/// The status of executing a transaction. The VM decides whether or not we should `Keep` the
/// transaction output or `Discard` it based upon the execution of the transaction. We wrap these
/// decisions around a `VMStatus` that provides more detail on the final execution state of the VM.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TransactionStatus {
    /// Discard the transaction output
    Discard(DiscardedVMStatus),

    /// Keep the transaction output
    Keep(KeptVMStatus),
}

impl TransactionStatus {
    pub fn status(&self) -> Result<KeptVMStatus, StatusCode> {
        match self {
            TransactionStatus::Keep(status) => Ok(status.clone()),
            TransactionStatus::Discard(code) => Err(*code),
        }
    }

    pub fn is_discarded(&self) -> bool {
        match self {
            TransactionStatus::Discard(_) => true,
            TransactionStatus::Keep(_) => false,
        }
    }
}

impl From<VMStatus> for TransactionStatus {
    fn from(vm_status: VMStatus) -> Self {
        match vm_status.keep_or_discard() {
            Ok(recorded) => TransactionStatus::Keep(recorded),
            Err(code) => TransactionStatus::Discard(code),
        }
    }
}

/// The output of executing a transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionOutput {
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

    pub fn into_inner(self) -> (WriteSet, Vec<ContractEvent>, u64, TransactionStatus) {
        (self.write_set, self.events, self.gas_used, self.status)
    }
}

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct TransactionInfo {
    /// The hash of this transaction.
    pub transaction_hash: HashValue,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: HashValue,

    /// The amount of gas used.
    pub gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: KeptVMStatus,
}

impl TransactionInfo {
    /// Constructs a new `TransactionInfo` object using transaction hash, state root hash and event
    /// root hash.
    pub fn new(
        transaction_hash: HashValue,
        state_root_hash: HashValue,
        events: &[ContractEvent],
        gas_used: u64,
        status: KeptVMStatus,
    ) -> TransactionInfo {
        let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();
        let events_accumulator_hash =
            InMemoryAccumulator::from_leaves(event_hashes.as_slice()).root_hash();
        TransactionInfo {
            transaction_hash,
            state_root_hash,
            event_root_hash: events_accumulator_hash,
            gas_used,
            status,
        }
    }

    pub fn id(&self) -> HashValue {
        self.crypto_hash()
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

    pub fn status(&self) -> &KeptVMStatus {
        &self.status
    }
}

impl Sample for TransactionInfo {
    fn sample() -> Self {
        Self::new(
            SignedUserTransaction::sample().id(),
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            &[],
            0,
            KeptVMStatus::Executed,
        )
    }
}

/// `RichTransactionInfo` is a wrapper of `TransactionInfo` with more info,
/// such as `block_id`, `block_number` which is the block that include the txn producing the txn info.
/// We cannot put the block_id into txn_info, because txn_info is accumulated into block header.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RichTransactionInfo {
    pub block_id: HashValue,
    pub block_number: u64,
    pub transaction_info: TransactionInfo,
    /// Transaction index in block
    pub transaction_index: u32,
    /// Transaction global index in chain, equivalent to transaction accumulator's leaf index
    pub transaction_global_index: u64,
}

impl Deref for RichTransactionInfo {
    type Target = TransactionInfo;

    fn deref(&self) -> &Self::Target {
        &self.transaction_info
    }
}

impl RichTransactionInfo {
    pub fn new(
        block_id: HashValue,
        block_number: u64,
        transaction_info: TransactionInfo,
        transaction_index: u32,
        transaction_global_index: u64,
    ) -> Self {
        Self {
            block_id,
            block_number,
            transaction_info,
            transaction_index,
            transaction_global_index,
        }
    }

    pub fn block_id(&self) -> HashValue {
        self.block_id
    }

    pub fn txn_info(&self) -> &TransactionInfo {
        &self.transaction_info
    }
}

/// `Transaction` will be the transaction type used internally in the diem node to represent the
/// transaction to be processed and persisted.
///
/// We suppress the clippy warning here as we would expect most of the transaction to be user
/// transaction.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Transaction {
    /// Transaction submitted by the user. e.g: P2P payment transaction, publishing module
    /// transaction, etc.
    UserTransaction(SignedUserTransaction),
    /// Transaction to update the block metadata resource at the beginning of a block.
    BlockMetadata(BlockMetadata),
}

impl Transaction {
    pub fn as_signed_user_txn(&self) -> Result<&SignedUserTransaction> {
        match self {
            Transaction::UserTransaction(txn) => Ok(txn),
            _ => Err(format_err!("Not a user transaction.")),
        }
    }

    pub fn id(&self) -> HashValue {
        match self {
            Transaction::UserTransaction(signed) => signed.id(),
            Transaction::BlockMetadata(block_metadata) => block_metadata.id(),
        }
    }
}

impl TryFrom<Transaction> for SignedUserTransaction {
    type Error = Error;

    fn try_from(txn: Transaction) -> Result<Self> {
        match txn {
            Transaction::UserTransaction(txn) => Ok(txn),
            _ => Err(format_err!("Not a user transaction.")),
        }
    }
}

/// Pool transactions status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TxStatus {
    /// Added transaction
    Added,
    /// Rejected transaction
    Rejected,
    /// Dropped transaction
    Dropped,
    /// Invalid transaction
    Invalid,
    /// Canceled transaction
    Canceled,
    /// Culled transaction
    Culled,
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TxStatus::Added => "added",
            TxStatus::Rejected => "rejected",
            TxStatus::Dropped => "dropped",
            TxStatus::Invalid => "invalid",
            TxStatus::Canceled => "canceled",
            TxStatus::Culled => "culled",
        };
        write!(f, "{}", s)
    }
}
