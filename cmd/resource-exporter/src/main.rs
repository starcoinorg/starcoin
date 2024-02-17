use anyhow::format_err;
use clap::Parser;
use serde::{ser::SerializeMap, Serialize, Serializer};
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::Genesis;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_state_tree::StateTree;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::block::BlockNumber;
use starcoin_types::{
    access_path::DataType, account_state::AccountState, language_storage::StructTag,
};
use starcoin_vm_types::account_address::AccountAddress;
use std::fs::File;
use std::io::Write;
use std::{convert::TryInto, fmt::Debug, path::PathBuf, sync::Arc};

use starcoin_rpc_api::types::StrView;

#[derive(Serialize, Debug)]
pub struct AccountData<R: Serialize> {
    address: AccountAddress,
    #[serde(flatten)]
    resource: Option<R>,
}

pub fn export(
    from_dir: PathBuf,
    output: PathBuf,
    network: BuiltinNetworkID,
    block_number: Option<BlockNumber>,
    resource_struct_tag: StructTag,
) -> anyhow::Result<()> {
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        from_dir.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let cur_num = block_number.unwrap_or(chain_info.head().number());
    let block = chain
        .get_block_by_number(cur_num)?
        .ok_or_else(|| format_err!("get block by number {} error", cur_num))?;

    let root = block.header.state_root();
    let statedb = ChainStateDB::new(storage.clone(), Some(root));
    let value_annotator = MoveValueAnnotator::new(&statedb);

    let state_tree = StateTree::<AccountAddress>::new(storage.clone(), Some(root));

    let mut file = File::create(output)?;

    let global_states = state_tree.dump()?;

    use std::time::Instant;
    let now = Instant::now();
    let mut sum: u128 = 0;
    for (address_bytes, account_state_bytes) in global_states.iter() {
        let account: AccountAddress = bcs_ext::from_bytes(address_bytes)?;
        let account_state: AccountState = account_state_bytes.as_slice().try_into()?;
        let resource_root = account_state.storage_roots()[DataType::RESOURCE.storage_index()];
        let resource = match resource_root {
            None => None,
            Some(root) => {
                let account_tree = StateTree::<StructTag>::new(storage.clone(), Some(root));
                let data = account_tree.get(&resource_struct_tag)?;

                if let Some(d) = data {
                    let annotated_struct =
                        value_annotator.view_struct(resource_struct_tag.clone(), d.as_slice())?;
                    let resource = annotated_struct;
                    let resource_json_value = serde_json::to_value(MoveStruct(resource))?;
                    Some(resource_json_value)
                } else {
                    None
                }
            }
        };
        if let Some(res) = resource {
            let balance = (res
                .get("token")
                .unwrap()
                .get("value")
                .unwrap()
                .as_f64()
                .unwrap()
                / 1000000000.0) as u128;
            if balance > 0 {
                writeln!(file, "{} {}", account, balance)?;
                sum += balance;
            }
        }
    }
    println!("t2: {}", now.elapsed().as_millis());
    writeln!(file, "total {}", sum)?;
    file.flush()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct MoveStruct(AnnotatedMoveStruct);

impl serde::Serialize for MoveStruct {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.value.len()))?;
        for (field, value) in &self.0.value {
            map.serialize_entry(field.as_str(), &MoveValue(value.clone()))?;
        }
        map.end()
    }
}

#[derive(Debug, Clone)]
struct MoveValue(AnnotatedMoveValue);

impl serde::Serialize for MoveValue {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            AnnotatedMoveValue::Bool(b) => serializer.serialize_bool(*b),
            AnnotatedMoveValue::U8(v) => serializer.serialize_u8(*v),
            AnnotatedMoveValue::U64(v) => serializer.serialize_u64(*v),
            AnnotatedMoveValue::U128(v) => serializer.serialize_u128(*v),
            AnnotatedMoveValue::Address(v) => v.serialize(serializer),
            AnnotatedMoveValue::Vector(v) => {
                let vs: Vec<_> = v.clone().into_iter().map(MoveValue).collect();
                vs.serialize(serializer)
            }
            AnnotatedMoveValue::Bytes(v) => hex::encode(v).serialize(serializer),
            AnnotatedMoveValue::Struct(v) => MoveStruct(v.clone()).serialize(serializer),
            AnnotatedMoveValue::U16(v) => serializer.serialize_u16(*v),
            AnnotatedMoveValue::U32(v) => serializer.serialize_u32(*v),
            AnnotatedMoveValue::U256(v) => v.serialize(serializer),
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "resource-exporter", about = "onchain resource exporter")]
pub struct ExporterOptions {
    #[clap(long, short = 'n')]
    /// Chain Network, like main, proxima
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// output file, like accounts.csv
    pub output: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard
    pub db_path: PathBuf,

    #[clap(long, short = 'b')]
    pub block_number: Option<BlockNumber>,

    #[clap(
        help = "resource struct tag,",
        default_value = "0x1::Account::Balance<0x1::STC::STC>"
    )]
    resource_type: StrView<StructTag>,
}

fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    let guard = pprof::ProfilerGuard::new(100).unwrap();
    let option = ExporterOptions::parse();
    export(
        option.db_path,
        option.output,
        option.net,
        option.block_number,
        option.resource_type.0.clone(),
    )?;
    #[cfg(target_os = "linux")]
    if let Ok(report) = guard.report().build() {
        println!("ok, export graph");
        let file = std::fs::File::create("/tmp/flamegraph-resource-exporter-freq-100.svg").unwrap();
        report.flamegraph(file).unwrap();
    }
    Ok(())
}
