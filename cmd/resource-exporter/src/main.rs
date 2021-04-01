use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage};
use starcoin_types::access_path::AccessPath;
use starcoin_types::language_storage::{StructTag, TypeTag};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::parser::parse_type_tag;
use starcoin_vm_types::state_view::StateView;
use std::collections::BTreeSet;

use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;

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
    let storage = Storage::new(StorageInstance::new_db_instance(DBStorage::new(
        db,
        Default::default(),
    )?))?;
    let storage = Arc::new(storage);
    let mut block = storage
        .get_block(block_id)?
        .ok_or_else(|| anyhow::anyhow!("cannot not find block {}", block_id))?;
    let state_root = block.header.state_root;

    // find all minters.
    let authors = {
        let mut authors = BTreeSet::new();
        // ignore genesis block
        while block.header.number > 0 {
            authors.insert(block.header.author);
            let parent_block_id = block.header.parent_hash;
            // let parent_height = block.header.number - 1;
            block = storage
                .get_block(parent_block_id)?
                .ok_or_else(|| anyhow::anyhow!("cannot not find block {}", parent_block_id))?;
        }
        authors
    };

    let statedb = ChainStateDB::new(storage.clone(), Some(state_root));
    let value_annotator = MoveValueAnnotator::new(&statedb);

    let mut csv_writer = csv::WriterBuilder::new().from_path(output)?;

    // write csv header.
    {
        csv_writer.write_field("address")?;
        for f in fields {
            csv_writer.write_field(f)?;
        }
        csv_writer.write_record(None::<&[u8]>)?;
    }

    for address in &authors {
        let resource = statedb.get(&AccessPath::new(
            *address,
            AccessPath::resource_access_vec(&resource_struct_tag),
        ))?;
        let resource = match resource {
            Some(d) => {
                let annotated_struct =
                    value_annotator.view_struct(resource_struct_tag.clone(), d.as_slice())?;
                let resource = annotated_struct;
                let resource_json_value = serde_json::to_value(MoveStruct(resource))?;
                Some(resource_json_value)
            }
            None => None,
        };

        // write csv record.
        let record: Option<Vec<_>> = resource
            .as_ref()
            .map(|v| fields.iter().map(|f| v.pointer(f.as_str())).collect());
        if let Some(mut record) = record {
            let account_value = serde_json::to_value(address).unwrap();
            record.insert(0, Some(&account_value));
            csv_writer.serialize(record)?;
        }
    }

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
        }
    }
}

// fn json_index_chain<'a>(v: &'a serde_json::Value, index_chain: &str) -> &'a serde_json::Value {
//     let parts = index_chain.split(".");
//     let mut current = v;
//     for p in parts {
//         let idx: Option<usize> = p.parse().ok();
//         match idx {
//             Some(i) => current = current.index(i),
//             None => current = current.index(p),
//         }
//     }
//     current
// }

fn parse_struct_tag(input: &str) -> anyhow::Result<StructTag> {
    match parse_type_tag(input)? {
        TypeTag::Struct(s) => Ok(s),
        _ => {
            anyhow::bail!("invalid struct tag")
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "resource-exporter", about = "onchain resource exporter")]
pub struct ExporterOptions {
    #[structopt(long, short = "o", parse(from_os_str))]
    /// output file, like accounts.csv
    pub output: PathBuf,
    #[structopt(long, short = "i", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/proxima/starcoindb/db
    pub db_path: PathBuf,

    #[structopt(long)]
    /// block id which we want do snapshot on.
    pub block_id: HashValue,

    #[structopt(
        short="r",
        default_value = "0x1::Account::Balance<0x1::STC::STC>",
        parse(try_from_str=parse_struct_tag)
    )]
    /// resource struct tag.
    resource_type: StructTag,

    #[structopt(min_values = 1, required = true)]
    /// fields of the struct to output. it use pointer syntax of serde_json.
    /// like: /authentication_key /sequence_number /deposit_events/counter
    pub fields: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let option: ExporterOptions = ExporterOptions::from_args();
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
    Ok(())
}
