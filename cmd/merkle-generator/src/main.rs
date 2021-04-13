use merkle_generator::{encode, DataProof, Sha3Algorithm};
use merkletree::merkle::{next_pow2, MerkleTree};
use merkletree::store::VecStore;
use serde::Deserialize;
use starcoin_vm_types::account_address::AccountAddress;
use std::fmt::Debug;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "merkle-generator", about = "merkle proof generator")]
pub struct ExporterOptions {
    #[structopt(long, short, parse(from_os_str))]
    /// intput csv without header, like rewards.csv
    pub input: PathBuf,

    #[structopt(parse(from_os_str))]
    /// merkle output json file, like merkle.json
    pub output: PathBuf,
}

#[derive(Copy, Clone, Debug, Deserialize)]
struct InputData {
    address: AccountAddress,
    amount: u128,
}

fn main() -> anyhow::Result<()> {
    let option: ExporterOptions = ExporterOptions::from_args();
    let mut csv_reader = csv::ReaderBuilder::default()
        .has_headers(false)
        .from_path(option.input.as_path())?;

    let leaf_data = {
        let mut leafs = Vec::with_capacity(1024);
        for record in csv_reader.deserialize() {
            let data: InputData = record?;
            leafs.push(data);
        }
        leafs
    };

    let tree: MerkleTree<[u8; 32], Sha3Algorithm, VecStore<_>> = {
        let leaf_data_in_bytes: anyhow::Result<Vec<_>> = leaf_data
            .iter()
            .enumerate()
            .map(|(idx, data)| encode(idx as u64, data.address, data.amount))
            .collect();

        let mut leaf_data_in_bytes = leaf_data_in_bytes?;

        let empty_leafs = next_pow2(leaf_data.len()) - leaf_data.len();
        // fill with empty leafs with meaningless data.
        for i in 0..empty_leafs {
            let index = leaf_data.len() + i;
            leaf_data_in_bytes.push(encode(index as u64, AccountAddress::ZERO, 0u128)?);
        }
        merkletree::merkle::MerkleTree::from_data(leaf_data_in_bytes.into_iter())?
    };

    let root = format!("0x{}", hex::encode(tree.root()));

    let mut proofs = vec![];
    for (idx, leaf) in leaf_data.into_iter().enumerate() {
        let leaf_proof = tree.gen_proof(idx)?;
        let mut proof: Vec<_> = leaf_proof
            .lemma()
            .iter()
            .skip(1) //skip first.
            .map(|sibling| format!("0x{}", hex::encode(sibling)))
            .collect();

        // skip last.
        proof.pop();

        proofs.push(DataProof {
            address: leaf.address,
            amount: leaf.amount,
            index: idx as u64,
            proof,
        });
    }

    let output = serde_json::json!({
        "root": root,
        "proofs": proofs
    });
    let output_file = std::fs::OpenOptions::new()
        .create_new(true)
        .truncate(true)
        .write(true)
        .open(option.output.as_path())?;
    serde_json::to_writer_pretty(output_file, &output)?;
    println!(
        "Proof generated in {}, merkle root: {}",
        option.output.as_path().display(),
        root
    );
    Ok(())
}
