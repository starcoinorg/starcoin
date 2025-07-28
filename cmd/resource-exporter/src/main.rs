use clap::Parser;
use serde::{ser::SerializeMap, Serialize, Serializer};
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_state_tree::StateTree;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{
    db_storage::DBStorage, storage::StorageInstance, BlockStore, Storage, StorageVersion,
};
use starcoin_types::{
    access_path::DataType, account_state::AccountState, language_storage::StructTag,
};
use starcoin_vm_types::{account_address::AccountAddress, parser::parse_struct_tag};
use std::{
    convert::TryInto,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Serialize, Debug)]
pub struct AccountData<R: Serialize> {
    address: AccountAddress,
    #[serde(flatten)]
    resource: Option<R>,
}

pub fn export(
    db: &str,
    output: &Path,
    block_id: HashValue,
    resource_struct_tag: StructTag,
    fields: &[String],
) -> anyhow::Result<()> {
    let db_storage = DBStorage::open_with_cfs(
        "starcoindb",
        db,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage))?;
    let storage = Arc::new(storage);
    let block = storage
        .get_block(block_id)?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_id))?;

    let root = block.header.state_root();
    let statedb = ChainStateDB::new(storage.clone(), Some(root));
    let value_annotator = MoveValueAnnotator::new(&statedb);

    let state_tree = StateTree::<AccountAddress>::new(storage.clone(), Some(root));

    let mut csv_writer = csv::WriterBuilder::new().from_path(output)?;

    // write csv header.
    {
        csv_writer.write_field("address")?;
        for f in fields {
            csv_writer.write_field(f)?;
        }
        csv_writer.write_record(None::<&[u8]>)?;
    }

    let global_states = state_tree.dump()?;

    use std::time::Instant;
    let now = Instant::now();
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

        // write csv record.
        let record: Option<Vec<_>> = resource
            .as_ref()
            .map(|v| fields.iter().map(|f| v.pointer(f.as_str())).collect());
        if let Some(mut record) = record {
            let account_value = serde_json::to_value(account).unwrap();
            record.insert(0, Some(&account_value));
            csv_writer.serialize(record)?;
        }
    }
    println!("t2: {}", now.elapsed().as_millis());
    // flush csv writer
    csv_writer.flush()?;
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
    #[clap(long, short = 'o', parse(from_os_str))]
    /// output file, like accounts.csv
    pub output: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,

    #[clap(long)]
    /// block id which snapshot at.
    pub block_id: HashValue,

    #[clap(
        short='r',
        default_value = "0x1::Account::Balance<0x1::STC::STC>",
        parse(try_from_str=parse_struct_tag)
    )]
    /// resource struct tag.
    resource_type: StructTag,

    #[clap(min_values = 1, required = true)]
    /// fields of the struct to output. it use pointer syntax of serde_json.
    /// like: /authentication_key /sequence_number /deposit_events/counter
    pub fields: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    let guard = pprof::ProfilerGuard::new(100).unwrap();
    let option: ExporterOptions = ExporterOptions::parse();
    let output = option.output.as_path();
    let block_id = option.block_id;
    let resource = option.resource_type.clone();
    export(
        option.db_path.display().to_string().as_str(),
        output,
        block_id,
        resource,
        option.fields.as_slice(),
    )?;
    #[cfg(target_os = "linux")]
    if let Ok(report) = guard.report().build() {
        println!("ok, export graph");
        let file = std::fs::File::create("/tmp/flamegraph-resource-exporter-freq-100.svg").unwrap();
        report.flamegraph(file).unwrap();
    }
    Ok(())
}
