//! Writes valid encodings into `corpus/<target>/` so fuzzing starts from real message shapes.
//! Run from `fuzz/` without the fuzzing RUSTFLAGS: `cargo test --test seed_corpus -- --ignored`.

use anyhow::Result;
use bcs_ext::BCSCodec;
use network_api::messages::{
    Announcement, AnnouncementType, CompactBlockMessage, TransactionsMessage,
};
use network_p2p::business_layer_handle::BusinessLayerHandle;
use network_p2p::PeerId;
use serde::Serialize;
use starcoin_abi_resolver::ABIResolver;
use starcoin_crypto::HashValue;
use starcoin_fuzz::StdlibStateView;
use starcoin_network::network_p2p_handle::Networkp2pHandle;
use starcoin_network_rpc_api::{
    GetAccountState, GetBlockHeadersByNumber, GetBlockIds, GetStateWithProof, GetTableInfo,
    GetTxnsWithHash, GetTxnsWithSize, Ping,
};
use starcoin_rpc_api::types::{
    FunctionIdView, ModuleIdView, StructTagView, TransactionArgumentView, TypeTagView,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{Block, BlockBody};
use starcoin_types::compact_block::CompactBlock;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
use starcoin_vm_types::parser::{parse_type_tags, type_args_too_deep, MAX_TYPE_TAG_NESTING};
use starcoin_vm_types::transaction::{
    Module, Package, RawUserTransaction, Script, ScriptFunction, TransactionPayload,
};
use std::fs;
use std::path::Path;
use std::str::FromStr;

fn write(target: &str, name: &str, selector: Option<u8>, payload: Vec<u8>) -> Result<()> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join(target);
    fs::create_dir_all(&dir)?;
    let mut bytes = selector.map(|s| vec![s]).unwrap_or_default();
    bytes.extend(payload);
    fs::write(dir.join(name), bytes)?;
    Ok(())
}

fn bcs<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bcs_ext::to_bytes(value)
}

fn rpc_requests() -> Result<()> {
    let t = "rpc_request";
    let ping = Ping {
        msg: "ping".to_string(),
        err: false,
    };
    write(t, "ping", Some(0), bcs(&ping)?)?;
    write(
        t,
        "txns_size",
        Some(1),
        bcs(&GetTxnsWithSize { max_size: 10 })?,
    )?;
    let ids = vec![HashValue::random(), HashValue::random()];
    write(
        t,
        "txns_hash",
        Some(2),
        bcs(&GetTxnsWithHash { ids: ids.clone() })?,
    )?;
    let by_number = GetBlockHeadersByNumber::new(1000, 3, 10);
    write(t, "headers_by_number", Some(3), bcs(&by_number)?)?;
    let block_ids = GetBlockIds {
        start_number: 7,
        reverse: true,
        max_size: 10,
    };
    write(t, "block_ids", Some(4), bcs(&block_ids)?)?;
    write(t, "hashes", Some(5), bcs(&ids)?)?;
    write(t, "hash", Some(6), bcs(&ids[0])?)?;
    let account_state = GetAccountState {
        state_root: ids[0],
        account_address: AccountAddress::random(),
    };
    let state = GetStateWithProof {
        state_root: ids[0],
        access_path: AccessPath::from_str("0x1/1/0x1::Account::Balance<0x1::STC::STC>")?,
    };
    write(t, "state_with_proof", Some(8), bcs(&state)?)?;
    write(t, "account_state", Some(9), bcs(&account_state)?)?;
    write(
        t,
        "table_info",
        Some(11),
        bcs(&GetTableInfo(AccountAddress::random()))?,
    )?;
    Ok(())
}

fn p2p_notifications() -> Result<()> {
    let t = "p2p_notification";
    let txn = SignedUserTransaction::mock();
    write(
        t,
        "txns",
        Some(0),
        TransactionsMessage::new(vec![txn.clone()]).encode()?,
    )?;
    let status = ChainStatus::random();
    let block = Block::new(status.head().clone(), BlockBody::new(vec![txn], None));
    let compact = CompactBlockMessage::new(CompactBlock::new(block), status.info().clone());
    write(t, "compact_block", Some(1), compact.encode()?)?;
    let announcement = Announcement::new(AnnouncementType::Txn, vec![HashValue::random()]);
    write(t, "announcement", Some(2), announcement.encode()?)?;
    // The fuzz target's local node uses ChainInfo::default(), so this handshake passes the
    // genesis check and mutations reach the version and protocol checks.
    let chain_info = ChainInfo::default();
    let handshake = Networkp2pHandle::new(chain_info.clone()).get_generic_data()?;
    Networkp2pHandle::new(ChainInfo::default())
        .handshake(PeerId::random(), handshake.clone())
        .map_err(|e| anyhow::anyhow!("seed handshake rejected: {:?}", e))?;
    write(t, "handshake", Some(3), handshake)?;
    write(t, "chain_info", Some(4), chain_info.encode()?)?;
    write(t, "chain_status", Some(5), status.encode()?)?;
    Ok(())
}

/// `levels` nested `vector<…>` around `u8`.
fn nested_vector(levels: usize) -> TypeTag {
    (0..levels).fold(TypeTag::U8, |t, _| TypeTag::Vector(Box::new(t)))
}

fn signed_txns() -> Result<()> {
    let t = "signed_txn";
    let mock = SignedUserTransaction::mock();
    write(t, "mock", None, mock.encode()?)?;
    let function = |ty_args: Vec<TypeTag>| -> Result<ScriptFunction> {
        Ok(ScriptFunction::new(
            ModuleId::new(AccountAddress::ONE, Identifier::new("TransferScripts")?),
            Identifier::new("peer_to_peer_v2")?,
            ty_args,
            vec![bcs(&AccountAddress::ONE)?, bcs(&1u128)?],
        ))
    };
    let stdlib_module = {
        let mut code = vec![];
        stdlib::load_latest_compiled_modules()[0].serialize(&mut code)?;
        Module::new(code)
    };
    // The txpool admits type args nested up to MAX_TYPE_TAG_NESTING and rejects deeper ones.
    for (levels, name) in [
        (MAX_TYPE_TAG_NESTING, "at_limit"),
        (MAX_TYPE_TAG_NESTING + 1, "past_limit"),
    ] {
        let ty_args = vec![nested_vector(levels)];
        let payloads = [
            (
                "script",
                TransactionPayload::Script(Script::new(vec![], ty_args.clone(), vec![])),
            ),
            (
                "script_function",
                TransactionPayload::ScriptFunction(function(ty_args.clone())?),
            ),
            (
                "package",
                TransactionPayload::Package(Package::new(
                    vec![stdlib_module.clone()],
                    Some(function(ty_args.clone())?),
                )?),
            ),
        ];
        for (kind, payload) in payloads {
            let raw = RawUserTransaction::new_with_default_gas_token(
                AccountAddress::ONE,
                0,
                payload,
                10_000_000,
                1,
                u64::MAX,
                ChainId::test(),
            );
            let txn = SignedUserTransaction::new(raw, mock.authenticator());
            assert_eq!(
                type_args_too_deep(txn.payload().ty_args()),
                levels > MAX_TYPE_TAG_NESTING
            );
            write(t, &format!("{}_{}", kind, name), None, txn.encode()?)?;
        }
    }
    Ok(())
}

/// `levels` nested `0x1::Option::Option<…>` around `u8`.
fn nested_option(levels: usize) -> String {
    format!(
        "{}u8{}",
        "0x1::Option::Option<".repeat(levels),
        ">".repeat(levels)
    )
}

fn type_tags() -> Result<()> {
    let t = "type_tag_parser";
    let deep_option = nested_option(MAX_TYPE_TAG_NESTING);
    let deep_balance = format!(
        "0x1::Account::Balance<{}>",
        nested_option(MAX_TYPE_TAG_NESTING - 1)
    );
    let seeds: Vec<(u8, &str, String)> = vec![
        (
            0,
            "type_tag",
            "vector<0x1::Token::Token<0x1::STC::STC>>".to_string(),
        ),
        (0, "type_tag_deep", deep_option),
        (
            1,
            "struct_tag",
            "0x1::Account::Balance<0x1::STC::STC>".to_string(),
        ),
        (1, "struct_tag_deep", deep_balance.clone()),
        (1, "struct_tag_option8", nested_option(8)),
        (2, "type_tags", "u8, address, 0x1::STC::STC".to_string()),
        (3, "argument_u128", "100u128".to_string()),
        (3, "argument_bytes", "x\"0a0b\"".to_string()),
        (3, "argument_address", "0x1".to_string()),
        (
            4,
            "function_id",
            "0x1::TransferScripts::peer_to_peer_v2".to_string(),
        ),
        (5, "module_id", "0x1::Account".to_string()),
        (
            6,
            "access_path_resource",
            "0x1/1/0x1::Account::Account".to_string(),
        ),
        (6, "access_path_code", "0x1/0/Account".to_string()),
    ];
    for (selector, name, s) in &seeds {
        let accepted = match selector {
            0 => TypeTagView::from_str(s).is_ok(),
            1 => StructTagView::from_str(s).is_ok(),
            2 => parse_type_tags(s).is_ok(),
            3 => TransactionArgumentView::from_str(s).is_ok(),
            4 => FunctionIdView::from_str(s).is_ok(),
            5 => ModuleIdView::from_str(s).is_ok(),
            _ => AccessPath::from_str(s).is_ok(),
        };
        assert!(
            accepted,
            "seed {} is not accepted by its parser: {}",
            name, s
        );
        write(t, name, Some(*selector), s.as_bytes().to_vec())?;
    }
    // The struct seeds reach the resolver on both sides of its node budget: `Option<T>` holds a
    // `vector<T>`, so each level doubles the resolved tree.
    let state = StdlibStateView::load();
    let resolve = |s: &str| -> Result<()> {
        let tag = StructTagView::from_str(s)?;
        ABIResolver::new(&state).resolve_struct_tag(&tag.0)?;
        Ok(())
    };
    resolve("0x1::Account::Balance<0x1::STC::STC>")?;
    resolve(&nested_option(8))?;
    let err = resolve(&deep_balance).unwrap_err();
    assert!(err.to_string().contains("nodes"), "{}", err);
    Ok(())
}

#[test]
#[ignore = "writes seed files into corpus/"]
fn write_seed_corpus() -> Result<()> {
    rpc_requests()?;
    p2p_notifications()?;
    signed_txns()?;
    type_tags()
}
