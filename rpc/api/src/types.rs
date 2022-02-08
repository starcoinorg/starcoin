// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use bcs_ext::BCSCodec;
use hex::FromHex;
use jsonrpc_core_client::RpcChannel;
pub use node_api_types::*;
use schemars::{self, JsonSchema};
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Serializer};
use serde::{Deserializer, Serialize};
pub use starcoin_abi_decoder::DecodedMoveValue;
use starcoin_abi_decoder::{
    DecodedPackage, DecodedScript, DecodedScriptFunction, DecodedTransactionPayload,
};
use starcoin_abi_types::ModuleABI;
use starcoin_accumulator::proof::AccumulatorProof;
use starcoin_crypto::{CryptoMaterialError, HashValue, ValidCryptoMaterialStringExt};
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use starcoin_service_registry::ServiceRequest;
use starcoin_state_api::{StateProof, StateWithProof};
use starcoin_types::block::{
    Block, BlockBody, BlockHeader, BlockHeaderExtra, BlockInfo, BlockNumber,
};
use starcoin_types::contract_event::{ContractEvent, ContractEventInfo};
use starcoin_types::event::EventKey;
use starcoin_types::genesis_config;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_types::proof::SparseMerkleProof;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::authenticator::{AuthenticationKey, TransactionAuthenticator};
use starcoin_types::transaction::{RawUserTransaction, ScriptFunction, TransactionArgument};
use starcoin_types::vm_error::AbortLocation;
use starcoin_types::U256;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{parse_module_id, FunctionId, ModuleId, StructTag};
use starcoin_vm_types::parser::{parse_transaction_argument, parse_type_tag};
use starcoin_vm_types::sign_message::SignedMessage;
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;
use starcoin_vm_types::transaction::{
    RichTransactionInfo, Script, SignedUserTransaction, Transaction, TransactionInfo,
    TransactionOutput, TransactionPayload, TransactionStatus,
};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::vm_status::{DiscardedVMStatus, KeptVMStatus, StatusCode};
use starcoin_vm_types::write_set::WriteOp;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

pub type ByteCode = Vec<u8>;
mod node_api_types;
pub mod pubsub;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MintedBlockView {
    pub block_hash: HashValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceView {
    pub raw: StrView<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<DecodedMoveValue>,
}

impl From<Vec<u8>> for ResourceView {
    fn from(v: Vec<u8>) -> Self {
        Self {
            raw: StrView(v),
            json: None,
        }
    }
}

impl ResourceView {
    pub fn decode<R: MoveResource + DeserializeOwned>(&self) -> anyhow::Result<R> {
        bcs_ext::from_bytes(self.raw.0.as_slice())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeView {
    pub code: StrView<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ModuleABI>,
}

impl From<Vec<u8>> for CodeView {
    fn from(v: Vec<u8>) -> Self {
        Self {
            code: StrView(v),
            abi: None,
        }
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListResourceView {
    pub resources: BTreeMap<StructTagView, ResourceView>,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListCodeView {
    #[schemars(with = "String")]
    pub codes: BTreeMap<Identifier, CodeView>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AnnotatedMoveStructView {
    pub abilities: u8,
    pub type_: StructTagView,
    #[schemars(with = "Vec<(String, AnnotatedMoveValueView)>")]
    pub value: Vec<(Identifier, AnnotatedMoveValueView)>,
}

impl From<AnnotatedMoveStruct> for AnnotatedMoveStructView {
    fn from(origin: AnnotatedMoveStruct) -> Self {
        Self {
            abilities: origin.abilities.into_u8(),
            type_: StrView(origin.type_),
            value: origin
                .value
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub enum AnnotatedMoveValueView {
    U8(u8),
    U64(StrView<u64>),
    U128(StrView<u128>),
    Bool(bool),
    Address(AccountAddress),
    Vector(Vec<AnnotatedMoveValueView>),
    Bytes(StrView<Vec<u8>>),
    Struct(AnnotatedMoveStructView),
}

impl From<AnnotatedMoveValue> for AnnotatedMoveValueView {
    fn from(origin: AnnotatedMoveValue) -> Self {
        match origin {
            AnnotatedMoveValue::U8(u) => AnnotatedMoveValueView::U8(u),
            AnnotatedMoveValue::U64(u) => AnnotatedMoveValueView::U64(StrView(u)),
            AnnotatedMoveValue::U128(u) => AnnotatedMoveValueView::U128(StrView(u)),
            AnnotatedMoveValue::Bool(b) => AnnotatedMoveValueView::Bool(b),
            AnnotatedMoveValue::Address(data) => AnnotatedMoveValueView::Address(data),
            AnnotatedMoveValue::Vector(data) => {
                AnnotatedMoveValueView::Vector(data.into_iter().map(Into::into).collect())
            }
            AnnotatedMoveValue::Bytes(data) => AnnotatedMoveValueView::Bytes(StrView(data)),
            AnnotatedMoveValue::Struct(data) => AnnotatedMoveValueView::Struct(data.into()),
        }
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct AccountStateSetView {
    #[schemars(with = "String")] //TODO impl in schemars
    pub codes: BTreeMap<Identifier, StrView<ByteCode>>,
    pub resources: BTreeMap<StructTagView, AnnotatedMoveStructView>,
}

#[derive(Default, Clone, Debug, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct TransactionRequest {
    /// Sender's address.
    #[schemars(with = "Option<String>")]
    pub sender: Option<AccountAddress>,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: Option<u64>,
    /// The transaction script to execute.
    #[serde(default)]
    pub script: Option<ScriptData>,
    /// module codes.
    #[serde(default)]
    pub modules: Vec<StrView<ByteCode>>,
    // Maximal total gas specified by wallet to spend for this transaction.
    pub max_gas_amount: Option<u64>,
    // Maximal price can be paid per gas.
    pub gas_unit_price: Option<u64>,
    // The token code for pay transaction gas, Default is STC token code.
    pub gas_token_code: Option<String>,
    // Expiration timestamp for this transaction. timestamp is represented
    // as u64 in seconds from Unix Epoch. If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    pub expiration_timestamp_secs: Option<u64>,
    pub chain_id: Option<u8>,
}

impl From<RawUserTransaction> for TransactionRequest {
    fn from(raw: RawUserTransaction) -> Self {
        let mut request = TransactionRequest {
            sender: Some(raw.sender()),
            sequence_number: Some(raw.sequence_number()),
            script: None,
            modules: vec![],
            max_gas_amount: Some(raw.max_gas_amount()),
            gas_unit_price: Some(raw.gas_unit_price()),
            gas_token_code: Some(raw.gas_token_code()),
            expiration_timestamp_secs: Some(raw.expiration_timestamp_secs()),
            chain_id: Some(raw.chain_id().id()),
        };
        match raw.into_payload() {
            TransactionPayload::Script(s) => {
                request.script = Some(s.into());
            }
            TransactionPayload::Package(p) => {
                let (_, m, s) = p.into_inner();
                request.script = s.map(Into::into);
                request.modules = m.into_iter().map(|m| StrView(m.into())).collect();
            }
            TransactionPayload::ScriptFunction(s) => {
                request.script = Some(ScriptData::from(s));
            }
        }
        request
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct DryRunTransactionRequest {
    #[serde(flatten)]
    pub transaction: TransactionRequest,
    /// Sender's public key
    pub sender_public_key: StrView<AccountPublicKey>,
}

#[derive(Clone, Debug, Eq, PartialEq, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub enum ArgumentsView {
    HumanReadable(Vec<TransactionArgumentView>),
    BCS(Vec<StrView<Vec<u8>>>),
}

impl ArgumentsView {
    pub fn to_bcs_bytes(&self) -> Vec<Vec<u8>> {
        match self {
            Self::HumanReadable(vs) => {
                convert_txn_args(&vs.iter().map(|v| v.0.clone()).collect::<Vec<_>>())
            }
            Self::BCS(vs) => vs.iter().map(|v| v.0.clone()).collect(),
        }
    }
}

/// Be caution:
/// We only allow passing args by TransactionArgumentView to our jsonrpc.
/// Because we cannot distinguish whether `0x12341235` is an human readable address or just some bcs bytes in hex string.
impl<'de> Deserialize<'de> for ArgumentsView {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let args = <Vec<TransactionArgumentView>>::deserialize(deserializer)?;
        Ok(ArgumentsView::HumanReadable(args))
    }
}

/// Only return BCS hex string when returning arguments out of jsonrpc.
impl Serialize for ArgumentsView {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            Self::HumanReadable(_vs) => {
                // transform view to bcs first.
                let data: Vec<_> = self.to_bcs_bytes().into_iter().map(StrView).collect();
                data.serialize(serializer)
            }
            Self::BCS(data) => data.serialize(serializer),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct ScriptData {
    pub code: StrView<ByteCodeOrScriptFunction>,
    #[serde(default)]
    pub type_args: Vec<TypeTagView>,
    pub args: ArgumentsView,
}

impl ScriptData {
    pub fn into_script_function(self) -> anyhow::Result<ScriptFunction> {
        match self.into_data() {
            Err(script_function) => Ok(script_function),
            _ => {
                anyhow::bail!("not a script function");
            }
        }
    }
    pub fn into_script(self) -> anyhow::Result<Script> {
        match self.into_data() {
            Ok(script) => Ok(script),
            _ => {
                anyhow::bail!("not a script");
            }
        }
    }
    fn into_data(self) -> Result<Script, ScriptFunction> {
        let ty_args: Vec<_> = self.type_args.into_iter().map(|s| s.0).collect();
        let args: Vec<_> = self.args.to_bcs_bytes();

        match self.code.0 {
            ByteCodeOrScriptFunction::ByteCode(code) => Ok(Script::new(code, ty_args, args)),
            ByteCodeOrScriptFunction::ScriptFunction(FunctionId { module, function }) => {
                Err(ScriptFunction::new(module, function, ty_args, args))
            }
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<TransactionPayload> for ScriptData {
    fn into(self) -> TransactionPayload {
        match self.into_data() {
            Ok(script) => TransactionPayload::Script(script),
            Err(func) => TransactionPayload::ScriptFunction(func),
        }
    }
}

impl From<Script> for ScriptData {
    fn from(s: Script) -> Self {
        let (code, ty_args, args) = s.into_inner();
        ScriptData {
            code: StrView(ByteCodeOrScriptFunction::ByteCode(code)),
            type_args: ty_args.into_iter().map(TypeTagView::from).collect(),
            args: ArgumentsView::BCS(args.into_iter().map(StrView).collect()),
        }
    }
}

impl From<ScriptFunction> for ScriptData {
    fn from(s: ScriptFunction) -> Self {
        let (module, function, ty_args, args) = s.into_inner();
        ScriptData {
            code: StrView(ByteCodeOrScriptFunction::ScriptFunction(FunctionId {
                module,
                function,
            })),
            type_args: ty_args.into_iter().map(TypeTagView::from).collect(),
            args: ArgumentsView::BCS(args.into_iter().map(StrView).collect()),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum ByteCodeOrScriptFunction {
    ByteCode(ByteCode),
    ScriptFunction(FunctionId),
}

impl std::fmt::Display for ByteCodeOrScriptFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ByteCodeOrScriptFunction::ByteCode(c) => write!(f, "0x{}", hex::encode(c)),
            ByteCodeOrScriptFunction::ScriptFunction(FunctionId { module, function }) => {
                write!(f, "{}::{}", module, function)
            }
        }
    }
}

impl FromStr for ByteCodeOrScriptFunction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits: Vec<&str> = s.rsplitn(2, "::").collect();
        if splits.len() == 2 {
            let module_id = ModuleIdView::from_str(splits[1])?;
            let function = Identifier::new(splits[0])?;
            Ok(ByteCodeOrScriptFunction::ScriptFunction(FunctionId {
                module: module_id.0,
                function,
            }))
        } else {
            Ok(ByteCodeOrScriptFunction::ByteCode(hex::decode(
                s.strip_prefix("0x").unwrap_or(s),
            )?))
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockHeaderView {
    pub block_hash: HashValue,
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: StrView<u64>,
    /// Block number.
    pub number: StrView<BlockNumber>,
    /// Block author.
    pub author: AccountAddress,
    /// Block author auth key.
    pub author_auth_key: Option<AuthenticationKey>,
    /// The transaction accumulator root hash after executing this block.
    pub txn_accumulator_root: HashValue,
    /// The block accumulator root hash.
    pub block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: StrView<u64>,
    /// Block difficulty
    #[schemars(with = "String")]
    pub difficulty: U256,
    /// hash for block body
    pub body_hash: HashValue,
    /// The chain id
    pub chain_id: u8,
    /// Consensus nonce field.
    pub nonce: u32,
    /// block header extra
    pub extra: BlockHeaderExtra,
}

impl From<BlockHeader> for BlockHeaderView {
    fn from(origin: BlockHeader) -> Self {
        BlockHeaderView {
            block_hash: origin.id(),
            parent_hash: origin.parent_hash(),
            timestamp: origin.timestamp().into(),
            number: origin.number().into(),
            author: origin.author(),
            author_auth_key: origin.author_auth_key(),
            txn_accumulator_root: origin.txn_accumulator_root(),
            block_accumulator_root: origin.block_accumulator_root(),
            state_root: origin.state_root(),
            gas_used: origin.gas_used().into(),
            difficulty: origin.difficulty(),
            body_hash: origin.body_hash(),
            chain_id: origin.chain_id().id(),
            nonce: origin.nonce(),
            extra: *origin.extra(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RawUserTransactionView {
    /// Sender's address.
    pub sender: AccountAddress,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: StrView<u64>,

    // The transaction payload in bcs_ext bytes.
    pub payload: StrView<Vec<u8>>,
    // decoded transaction payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decoded_payload: Option<TransactionPayloadView>,
    // Maximal total gas specified by wallet to spend for this transaction.
    pub max_gas_amount: StrView<u64>,
    // Maximal price can be paid per gas.
    pub gas_unit_price: StrView<u64>,
    // The token code for pay transaction gas, Default is STC token code.
    pub gas_token_code: String,
    // Expiration timestamp for this transaction. timestamp is represented
    // as u64 in seconds from Unix Epoch. If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    pub expiration_timestamp_secs: StrView<u64>,
    pub chain_id: u8,
}

impl TryFrom<RawUserTransaction> for RawUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(origin: RawUserTransaction) -> Result<Self, Self::Error> {
        Ok(RawUserTransactionView {
            sender: origin.sender(),
            sequence_number: origin.sequence_number().into(),
            max_gas_amount: origin.max_gas_amount().into(),
            gas_unit_price: origin.gas_unit_price().into(),
            gas_token_code: origin.gas_token_code(),
            expiration_timestamp_secs: origin.expiration_timestamp_secs().into(),
            chain_id: origin.chain_id().id(),
            payload: StrView(origin.into_payload().encode()?),
            decoded_payload: None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TransactionPayloadView {
    /// A transaction that executes code.
    Script(DecodedScriptView),
    /// A transaction that publish or update module code by a package.
    Package(DecodedPackageView),
    /// A transaction that executes an existing script function published on-chain.
    ScriptFunction(DecodedScriptFunctionView),
}

impl From<DecodedTransactionPayload> for TransactionPayloadView {
    fn from(orig: DecodedTransactionPayload) -> Self {
        match orig {
            DecodedTransactionPayload::Script(s) => TransactionPayloadView::Script(s.into()),
            DecodedTransactionPayload::Package(p) => TransactionPayloadView::Package(p.into()),
            DecodedTransactionPayload::ScriptFunction(s) => {
                TransactionPayloadView::ScriptFunction(s.into())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecodedScriptView {
    pub code: StrView<Vec<u8>>,
    pub ty_args: Vec<TypeTagView>,
    pub args: Vec<DecodedMoveValue>,
}

impl From<DecodedScript> for DecodedScriptView {
    fn from(orig: DecodedScript) -> Self {
        Self {
            code: StrView(orig.code),
            ty_args: orig.ty_args.into_iter().map(StrView).collect(),
            args: orig.args,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecodedPackageView {
    pub package_address: AccountAddress,
    pub modules: Vec<StrView<Vec<u8>>>,
    pub init_script: Option<DecodedScriptFunctionView>,
}

impl From<DecodedPackage> for DecodedPackageView {
    fn from(orig: DecodedPackage) -> Self {
        Self {
            package_address: orig.package_address,
            modules: orig
                .modules
                .into_iter()
                .map(|m| StrView(m.code().to_vec()))
                .collect(),
            init_script: orig.init_script.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecodedScriptFunctionView {
    pub module: ModuleIdView,
    #[schemars(with = "String")]
    pub function: Identifier,
    pub ty_args: Vec<TypeTagView>,
    pub args: Vec<DecodedMoveValue>,
}

impl From<DecodedScriptFunction> for DecodedScriptFunctionView {
    fn from(orig: DecodedScriptFunction) -> Self {
        Self {
            module: StrView(orig.module),
            function: orig.function,
            ty_args: orig.ty_args.into_iter().map(StrView).collect(),
            args: orig.args,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SignedUserTransactionView {
    pub transaction_hash: HashValue,
    /// The raw transaction
    pub raw_txn: RawUserTransactionView,

    /// Public key and signature to authenticate
    pub authenticator: TransactionAuthenticator,
}

impl TryFrom<SignedUserTransaction> for SignedUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(txn: SignedUserTransaction) -> Result<Self, Self::Error> {
        let auth = txn.authenticator();
        let txn_hash = txn.id();
        Ok(SignedUserTransactionView {
            transaction_hash: txn_hash,
            raw_txn: txn.into_raw_transaction().try_into()?,
            authenticator: auth,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct BlockMetadataView {
    /// Parent block hash.
    pub parent_hash: HashValue,
    pub timestamp: StrView<u64>,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    pub uncles: StrView<u64>,
    pub number: StrView<BlockNumber>,
    pub chain_id: u8,
    pub parent_gas_used: StrView<u64>,
}

impl From<BlockMetadata> for BlockMetadataView {
    fn from(origin: BlockMetadata) -> Self {
        let (
            parent_hash,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            chain_id,
            parent_gas_used,
        ) = origin.into_inner();
        BlockMetadataView {
            parent_hash,
            timestamp: timestamp.into(),
            author,
            author_auth_key,
            uncles: uncles.into(),
            number: number.into(),
            chain_id: chain_id.id(),
            parent_gas_used: parent_gas_used.into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<BlockMetadata> for BlockMetadataView {
    fn into(self) -> BlockMetadata {
        let BlockMetadataView {
            parent_hash,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            chain_id,
            parent_gas_used,
        } = self;
        BlockMetadata::new(
            parent_hash,
            timestamp.0,
            author,
            author_auth_key,
            uncles.0,
            number.0,
            genesis_config::ChainId::new(chain_id),
            parent_gas_used.0,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransactionView {
    pub block_hash: HashValue,
    pub block_number: StrView<BlockNumber>,
    pub transaction_hash: HashValue,
    pub transaction_index: u32,
    pub block_metadata: Option<BlockMetadataView>,
    pub user_transaction: Option<SignedUserTransactionView>,
}

impl TransactionView {
    pub fn new(txn: Transaction, block: &Block) -> anyhow::Result<Self> {
        let transaction_hash = txn.id();
        let block_hash = block.id();
        let block_number = block.header.number();
        let transaction_index = match &txn {
            Transaction::BlockMetadata(_) => 0,
            _ => {
                1 + block
                    .transactions()
                    .iter()
                    .position(|t| t.id() == transaction_hash)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "cannot find txn {} in block {}",
                            transaction_hash,
                            block_hash
                        )
                    })? as u32
            }
        };

        let (meta, txn) = match txn {
            Transaction::BlockMetadata(meta) => (Some(meta.into()), None),
            Transaction::UserTransaction(t) => (None, Some(t.try_into()?)),
        };
        Ok(Self {
            block_hash,
            block_number: block_number.into(),
            transaction_hash,
            transaction_index,
            block_metadata: meta,
            user_transaction: txn,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum BlockTransactionsView {
    Hashes(Vec<HashValue>),
    Full(Vec<SignedUserTransactionView>),
}

impl BlockTransactionsView {
    pub fn txn_hashes(&self) -> Vec<HashValue> {
        match self {
            Self::Hashes(h) => h.clone(),
            Self::Full(f) => f.iter().map(|t| t.transaction_hash).collect(),
        }
    }
}

impl TryFrom<Vec<SignedUserTransaction>> for BlockTransactionsView {
    type Error = anyhow::Error;

    fn try_from(txns: Vec<SignedUserTransaction>) -> Result<Self, Self::Error> {
        Ok(BlockTransactionsView::Full(
            txns.into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<Vec<HashValue>> for BlockTransactionsView {
    fn from(txns: Vec<HashValue>) -> Self {
        BlockTransactionsView::Hashes(txns)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockView {
    pub header: BlockHeaderView,
    pub body: BlockTransactionsView,
    pub uncles: Vec<BlockHeaderView>,
}

impl BlockView {
    pub fn try_from_block(block: Block, thin: bool) -> Result<Self, anyhow::Error> {
        let (header, body) = block.into_inner();
        let BlockBody {
            transactions,
            uncles,
        } = body;
        let txns_view = if thin {
            BlockTransactionsView::Hashes(transactions.into_iter().map(|t| t.id()).collect())
        } else {
            transactions.try_into()?
        };
        Ok(BlockView {
            header: header.into(),
            uncles: uncles
                .unwrap_or_default()
                .into_iter()
                .map(|h| h.into())
                .collect(),
            body: txns_view,
        })
    }
}

impl TryFrom<Block> for BlockView {
    type Error = anyhow::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        Self::try_from_block(block, false)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransactionInfoView {
    pub block_hash: HashValue,
    pub block_number: StrView<u64>,
    /// The hash of this transaction.
    pub transaction_hash: HashValue,
    /// The index of this transaction in block
    pub transaction_index: u32,
    /// The index of this transaction in chain
    pub transaction_global_index: StrView<u64>,
    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: HashValue,

    /// The amount of gas used.
    pub gas_used: StrView<u64>,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: TransactionStatusView,
}

impl TransactionInfoView {
    pub fn new(txn_info: RichTransactionInfo) -> Self {
        TransactionInfoView {
            block_hash: txn_info.block_id,
            block_number: txn_info.block_number.into(),
            transaction_hash: txn_info.transaction_hash,
            transaction_index: txn_info.transaction_index,
            transaction_global_index: txn_info.transaction_global_index.into(),
            state_root_hash: txn_info.transaction_info.state_root_hash,
            event_root_hash: txn_info.transaction_info.event_root_hash,
            gas_used: txn_info.transaction_info.gas_used.into(),
            status: TransactionStatusView::from(txn_info.transaction_info.status),
        }
    }
}

impl From<RichTransactionInfo> for TransactionInfoView {
    fn from(txn_info: RichTransactionInfo) -> Self {
        TransactionInfoView::new(txn_info)
    }
}

impl TryFrom<TransactionInfoView> for RichTransactionInfo {
    type Error = anyhow::Error;

    fn try_from(view: TransactionInfoView) -> Result<Self, Self::Error> {
        let status: TransactionStatus = view.status.clone().into();
        match status {
            TransactionStatus::Keep(kept_status) => Ok(RichTransactionInfo::new(
                view.block_hash,
                view.block_number.0,
                TransactionInfo {
                    transaction_hash: view.transaction_hash,

                    state_root_hash: view.state_root_hash,
                    event_root_hash: view.event_root_hash,
                    gas_used: view.gas_used.0,
                    status: kept_status,
                },
                view.transaction_index,
                view.transaction_global_index.0,
            )),
            TransactionStatus::Discard(_) => {
                bail!("TransactionInfoView's status is discard, {:?}, can not convert to RichTransactionInfo", view);
            }
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub enum TransactionStatusView {
    Executed,
    OutOfGas,

    MoveAbort {
        //Todo: remote define it
        #[schemars(with = "String")]
        location: AbortLocation,
        abort_code: StrView<u64>,
    },
    ExecutionFailure {
        #[schemars(with = "String")]
        location: AbortLocation,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError,
    Discard {
        status_code: StrView<u64>,
        status_code_name: String,
    },
}

impl From<TransactionStatus> for TransactionStatusView {
    fn from(s: TransactionStatus) -> Self {
        match s {
            TransactionStatus::Discard(d) => d.into(),
            TransactionStatus::Keep(k) => k.into(),
        }
    }
}

impl From<KeptVMStatus> for TransactionStatusView {
    fn from(origin: KeptVMStatus) -> Self {
        match origin {
            KeptVMStatus::Executed => TransactionStatusView::Executed,
            KeptVMStatus::OutOfGas => TransactionStatusView::OutOfGas,
            KeptVMStatus::MoveAbort(l, c) => TransactionStatusView::MoveAbort {
                location: l,
                abort_code: c.into(),
            },
            KeptVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
            } => TransactionStatusView::ExecutionFailure {
                location,
                function,
                code_offset,
            },
            KeptVMStatus::MiscellaneousError => TransactionStatusView::MiscellaneousError,
        }
    }
}

impl From<DiscardedVMStatus> for TransactionStatusView {
    fn from(s: DiscardedVMStatus) -> Self {
        Self::Discard {
            status_code: StrView(s.into()),
            status_code_name: format!("{:?}", s),
        }
    }
}

impl From<TransactionStatusView> for TransactionStatus {
    fn from(value: TransactionStatusView) -> Self {
        match value {
            TransactionStatusView::Executed => TransactionStatus::Keep(KeptVMStatus::Executed),
            TransactionStatusView::OutOfGas => TransactionStatus::Keep(KeptVMStatus::OutOfGas),
            TransactionStatusView::MoveAbort {
                location,
                abort_code,
            } => TransactionStatus::Keep(KeptVMStatus::MoveAbort(location, abort_code.0)),
            TransactionStatusView::MiscellaneousError => {
                TransactionStatus::Keep(KeptVMStatus::MiscellaneousError)
            }
            TransactionStatusView::ExecutionFailure {
                location,
                function,
                code_offset,
            } => TransactionStatus::Keep(KeptVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
            }),
            TransactionStatusView::Discard {
                status_code,
                status_code_name: _,
            } => TransactionStatus::Discard(
                status_code
                    .0
                    .try_into()
                    .ok()
                    .unwrap_or(StatusCode::UNKNOWN_STATUS),
            ),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct TransactionEventResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode_event_data: Option<DecodedMoveValue>,
    #[serde(flatten)]
    pub event: TransactionEventView,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
pub struct TransactionEventView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<HashValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<StrView<BlockNumber>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_global_index: Option<StrView<u64>>,
    pub data: StrView<Vec<u8>>,
    pub type_tag: TypeTagView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_index: Option<u32>,
    pub event_key: EventKey,
    pub event_seq_number: StrView<u64>,
}

impl From<ContractEventInfo> for TransactionEventView {
    fn from(info: ContractEventInfo) -> Self {
        TransactionEventView {
            block_hash: Some(info.block_hash),
            block_number: Some(info.block_number.into()),
            transaction_hash: Some(info.transaction_hash),
            transaction_index: Some(info.transaction_index),
            transaction_global_index: Some(info.transaction_global_index.into()),
            data: StrView(info.event.event_data().to_vec()),
            type_tag: info.event.type_tag().clone().into(),
            event_index: Some(info.event_index),
            event_key: *info.event.key(),
            event_seq_number: info.event.sequence_number().into(),
        }
    }
}

impl From<ContractEvent> for TransactionEventView {
    fn from(event: ContractEvent) -> Self {
        TransactionEventView {
            block_hash: None,
            block_number: None,
            transaction_hash: None,
            transaction_index: None,
            transaction_global_index: None,
            data: StrView(event.event_data().to_vec()),
            type_tag: event.type_tag().clone().into(),
            event_index: None,
            event_key: *event.key(),
            event_seq_number: event.sequence_number().into(),
        }
    }
}

impl TransactionEventView {
    pub fn new(
        block_hash: Option<HashValue>,
        block_number: Option<BlockNumber>,
        transaction_hash: Option<HashValue>,
        transaction_index: Option<u32>,
        transaction_global_index: Option<u64>,
        event_index: Option<u32>,
        contract_event: &ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number: block_number.map(Into::into),
            transaction_hash,
            transaction_index,
            transaction_global_index: transaction_global_index.map(Into::into),
            data: StrView(contract_event.event_data().to_vec()),
            type_tag: contract_event.type_tag().clone().into(),
            event_index,
            event_key: *contract_event.key(),
            event_seq_number: contract_event.sequence_number().into(),
        }
    }
}

use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_chain_api::{EventWithProof, TransactionInfoWithProof};
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::move_resource::MoveResource;
pub use vm_status_translator::VmStatusExplainView;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DryRunOutputView {
    pub explained_status: VmStatusExplainView,
    #[serde(flatten)]
    pub txn_output: TransactionOutputView,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionOutputView {
    pub status: TransactionStatusView,
    pub gas_used: StrView<u64>,
    pub write_set: Vec<TransactionOutputAction>,
    pub events: Vec<TransactionEventView>,
}

impl From<TransactionOutput> for TransactionOutputView {
    fn from(txn_output: TransactionOutput) -> Self {
        let (write_set, events, gas_used, status) = txn_output.into_inner();
        Self {
            events: events.into_iter().map(Into::into).collect(),
            gas_used: gas_used.into(),
            status: status.into(),
            write_set: write_set
                .into_iter()
                .map(TransactionOutputAction::from)
                .collect(),
        }
    }
}
impl From<(AccessPath, WriteOp)> for TransactionOutputAction {
    fn from((access_path, op): (AccessPath, WriteOp)) -> Self {
        let (action, value) = match op {
            WriteOp::Deletion => (WriteOpView::Deletion, None),
            WriteOp::Value(v) => (
                WriteOpView::Value,
                Some(if access_path.path.is_resource() {
                    WriteOpValueView::Resource(v.into())
                } else {
                    WriteOpValueView::Code(v.into())
                }),
            ),
        };

        TransactionOutputAction {
            access_path,
            action,
            value,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionOutputAction {
    pub access_path: AccessPath,
    pub action: WriteOpView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<WriteOpValueView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum WriteOpValueView {
    Code(CodeView),
    Resource(ResourceView),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum WriteOpView {
    Deletion,
    Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChainInfoView {
    pub chain_id: u8,
    pub genesis_hash: HashValue,
    pub head: BlockHeaderView,
    //TODO should define block info view?
    pub block_info: BlockInfoView,
}

impl From<ChainInfo> for ChainInfoView {
    fn from(info: ChainInfo) -> Self {
        let (chain_id, genesis_hash, status) = info.into_inner();
        let (head, block_info) = status.into_inner();
        Self {
            chain_id: chain_id.into(),
            genesis_hash,
            head: head.into(),
            block_info: block_info.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PeerInfoView {
    pub peer_id: PeerId,
    pub chain_info: ChainInfoView,
    pub notif_protocols: String,
    pub rpc_protocols: String,
}

impl From<PeerInfo> for PeerInfoView {
    fn from(info: PeerInfo) -> Self {
        Self {
            peer_id: info.peer_id,
            chain_info: info.chain_info.into(),
            notif_protocols: info.notif_protocols.join(","),
            rpc_protocols: info.rpc_protocols.join(","),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SparseMerkleProofView {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(HashValue, HashValue)`
    ///         - If the first `HashValue` equals requested key, this is an inclusion proof and the
    ///           second `HashValue` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. The first `HashValue` is the only key
    ///           that exists in the subtree and the second `HashValue` equals the hash of the
    ///           corresponding account blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    pub leaf: Option<(HashValue, HashValue)>,

    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    pub siblings: Vec<HashValue>,
}

impl From<SparseMerkleProof> for SparseMerkleProofView {
    fn from(origin: SparseMerkleProof) -> Self {
        Self {
            leaf: origin.leaf,
            siblings: origin.siblings,
        }
    }
}

impl From<SparseMerkleProofView> for SparseMerkleProof {
    fn from(origin: SparseMerkleProofView) -> Self {
        Self {
            leaf: origin.leaf,
            siblings: origin.siblings,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StateWithProofView {
    pub state: Option<StrView<Vec<u8>>>,
    pub account_state: Option<StrView<Vec<u8>>>,
    pub account_proof: SparseMerkleProofView,
    pub account_state_proof: SparseMerkleProofView,
}

impl StateWithProofView {
    pub fn into_state_proof(self) -> StateWithProof {
        self.into()
    }
}

impl From<StateWithProof> for StateWithProofView {
    fn from(state_proof: StateWithProof) -> Self {
        let state = state_proof.state.map(StrView);
        Self {
            state,
            account_state: state_proof.proof.account_state.map(|b| StrView(b.into())),
            account_proof: state_proof.proof.account_proof.into(),
            account_state_proof: state_proof.proof.account_state_proof.into(),
        }
    }
}

impl From<StateWithProofView> for StateWithProof {
    fn from(view: StateWithProofView) -> Self {
        let state = view.state.map(|v| v.0);
        let proof = StateProof::new(
            view.account_state.map(|v| v.0),
            view.account_proof.into(),
            view.account_state_proof.into(),
        );
        StateWithProof::new(state, proof)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccumulatorProofView {
    pub siblings: Vec<HashValue>,
}

impl From<AccumulatorProof> for AccumulatorProofView {
    fn from(origin: AccumulatorProof) -> Self {
        Self {
            siblings: origin.siblings,
        }
    }
}

impl From<AccumulatorProofView> for AccumulatorProof {
    fn from(view: AccumulatorProofView) -> Self {
        Self {
            siblings: view.siblings,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventWithProofView {
    /// event is serialized bytes in bcs format.
    pub event: StrView<Vec<u8>>,
    pub proof: AccumulatorProofView,
}

impl From<EventWithProof> for EventWithProofView {
    fn from(origin: EventWithProof) -> Self {
        Self {
            event: StrView(origin.event.encode().expect("encode event should success")),
            proof: origin.proof.into(),
        }
    }
}

impl TryFrom<EventWithProofView> for EventWithProof {
    type Error = anyhow::Error;

    fn try_from(value: EventWithProofView) -> Result<Self, Self::Error> {
        Ok(EventWithProof {
            event: ContractEvent::decode(value.event.0.as_slice())?,
            proof: value.proof.into(),
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionInfoWithProofView {
    pub transaction_info: TransactionInfoView,
    pub proof: AccumulatorProofView,
    pub event_proof: Option<EventWithProofView>,
    pub state_proof: Option<StateWithProofView>,
}

impl From<TransactionInfoWithProof> for TransactionInfoWithProofView {
    fn from(origin: TransactionInfoWithProof) -> Self {
        Self {
            transaction_info: origin.transaction_info.into(),
            proof: origin.proof.into(),
            event_proof: origin.event_proof.map(Into::into),
            state_proof: origin.state_proof.map(Into::into),
        }
    }
}

impl TryFrom<TransactionInfoWithProofView> for TransactionInfoWithProof {
    type Error = anyhow::Error;

    fn try_from(view: TransactionInfoWithProofView) -> Result<Self, Self::Error> {
        Ok(TransactionInfoWithProof {
            transaction_info: view.transaction_info.try_into()?,
            proof: view.proof.into(),
            event_proof: view.event_proof.map(TryInto::try_into).transpose()?,
            state_proof: view.state_proof.map(Into::into),
        })
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct StrView<T>(pub T);

impl<T> JsonSchema for StrView<T> {
    fn schema_name() -> String {
        std::any::type_name::<T>().to_owned()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            ..Default::default()
        }
        .into()
    }
}

impl<T> From<T> for StrView<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T> Serialize for StrView<T>
where
    Self: ToString,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> Deserialize<'de> for StrView<T>
where
    Self: FromStr,
    <Self as FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;

        StrView::<T>::from_str(&s).map_err(D::Error::custom)
    }
}

pub type SignedMessageView = StrView<SignedMessage>;

impl FromStr for SignedMessageView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(SignedMessage::from_str(s)?))
    }
}

impl std::fmt::Display for SignedMessageView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

//TODO auto implement FromStr for StrView<T> where T:FromStr

pub type ModuleIdView = StrView<ModuleId>;
pub type TypeTagView = StrView<TypeTag>;
pub type StructTagView = StrView<StructTag>;
pub type TransactionArgumentView = StrView<TransactionArgument>;
pub type FunctionIdView = StrView<FunctionId>;

impl std::fmt::Display for FunctionIdView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for FunctionIdView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(FunctionId::from_str(s)?))
    }
}

impl std::fmt::Display for StrView<ModuleId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for StrView<ModuleId> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_module_id(s)?))
    }
}

impl std::fmt::Display for TypeTagView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for TypeTagView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        Ok(Self(type_tag))
    }
}

impl std::fmt::Display for StrView<StructTag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for StructTagView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        match type_tag {
            TypeTag::Struct(s) => Ok(Self(s)),
            t => anyhow::bail!("expect struct tag, actual: {}", t),
        }
    }
}

impl std::fmt::Display for StrView<TransactionArgument> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for TransactionArgumentView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arg = parse_transaction_argument(s)?;
        Ok(Self(arg))
    }
}

impl std::fmt::Display for StrView<Vec<u8>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for StrView<Vec<u8>> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s.strip_prefix("0x").unwrap_or(s))?))
    }
}

impl std::fmt::Display for StrView<AccountPublicKey> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_encoded_string().map_err(|_| std::fmt::Error)?
        )
    }
}

impl FromStr for StrView<AccountPublicKey> {
    type Err = CryptoMaterialError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AccountPublicKey::from_encoded_string(s).map(StrView)
    }
}

macro_rules! impl_str_view_for {
    ($($t:ty)*) => {$(
    impl std::fmt::Display for StrView<$t> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl FromStr for StrView<$t> {
        type Err = <$t as FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<$t>().map(StrView)
        }
    }
    impl From<StrView<$t>> for $t {
        fn from(view: StrView<$t>) -> $t {
            view.0
        }
    }
    )*}
}
impl_str_view_for! {u64 i64 u128 i128}
impl_str_view_for! {ByteCodeOrScriptFunction AccessPath}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BytesView(Box<[u8]>);

impl BytesView {
    pub fn new<T: Into<Box<[u8]>>>(bytes: T) -> Self {
        Self(bytes.into())
    }

    pub fn into_inner(self) -> Box<[u8]> {
        self.0
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for BytesView {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::AsRef<[u8]> for BytesView {
    fn as_ref(&self) -> &[u8] {
        self.inner()
    }
}

impl std::fmt::Display for BytesView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in self.inner() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl From<&[u8]> for BytesView {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<Vec<u8>> for BytesView {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes.into_boxed_slice())
    }
}

impl<'de> Deserialize<'de> for BytesView {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        <Vec<u8>>::from_hex(s)
            .map_err(D::Error::custom)
            .map(Into::into)
    }
}

impl Serialize for BytesView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex::encode(self).serialize(serializer)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ContractCall {
    pub function_id: FunctionIdView,
    pub type_args: Vec<TypeTagView>,
    pub args: Vec<TransactionArgumentView>,
}

#[derive(Debug, Clone)]
pub struct ConnectLocal;

impl ServiceRequest for ConnectLocal {
    type Response = RpcChannel;
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AccumulatorInfoView {
    /// Accumulator root hash
    pub accumulator_root: HashValue,
    /// Frozen subtree roots of this accumulator.
    pub frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in this accumulator.
    pub num_leaves: StrView<u64>,
    /// The total number of nodes in this accumulator.
    pub num_nodes: StrView<u64>,
}

impl From<AccumulatorInfo> for AccumulatorInfoView {
    fn from(info: AccumulatorInfo) -> Self {
        AccumulatorInfoView {
            accumulator_root: info.accumulator_root,
            frozen_subtree_roots: info.frozen_subtree_roots.clone(),
            num_leaves: info.num_leaves.into(),
            num_nodes: info.num_nodes.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockInfoView {
    /// Block hash
    pub block_hash: HashValue,
    /// The total difficulty.
    #[schemars(with = "String")]
    pub total_difficulty: U256,
    /// The transaction accumulator info
    pub txn_accumulator_info: AccumulatorInfoView,
    /// The block accumulator info.
    pub block_accumulator_info: AccumulatorInfoView,
}

impl From<BlockInfo> for BlockInfoView {
    fn from(block_info: BlockInfo) -> Self {
        BlockInfoView {
            block_hash: block_info.block_id,
            total_difficulty: block_info.total_difficulty,
            txn_accumulator_info: block_info.txn_accumulator_info.into(),
            block_accumulator_info: block_info.block_accumulator_info.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{ByteCodeOrScriptFunction, FunctionId};
    use starcoin_types::account_address::AccountAddress;

    #[test]
    fn test_script_data() {
        let script_function: ByteCodeOrScriptFunction = "0x1::M::func1".parse().unwrap();
        assert!(matches!(
            script_function,
            ByteCodeOrScriptFunction::ScriptFunction { .. }
        ));
        if let ByteCodeOrScriptFunction::ScriptFunction(FunctionId { module, function }) =
            script_function
        {
            assert_eq!(*module.address(), "0x1".parse::<AccountAddress>().unwrap());
            assert_eq!(module.name().as_str(), "M");
            assert_eq!(function.as_str(), "func1");
        }

        let bytecode: ByteCodeOrScriptFunction = "0x123432ab34".parse().unwrap();
        assert!(matches!(bytecode, ByteCodeOrScriptFunction::ByteCode(_)));
    }
}
