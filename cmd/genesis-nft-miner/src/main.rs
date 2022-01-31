use anyhow::Result;
use bcs_ext::BCSCodec;
use clap::Parser;
use jsonrpc_core_client::{RpcChannel, RpcError};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{HashValue, ValidCryptoMaterialStringExt};
use starcoin_rpc_api::types::{TransactionInfoView, TransactionStatusView};
use starcoin_rpc_api::{chain::ChainClient, state::StateClient, txpool::TxPoolClient};
use starcoin_types::access_path::{AccessPath, DataPath};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::{account_struct_tag, genesis_address, AccountResource};
use starcoin_types::genesis_config::ChainId;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::authenticator::AccountPrivateKey;
use starcoin_types::transaction::{RawUserTransaction, ScriptFunction, SignedUserTransaction};
use starcoin_vm_types::value::MoveValue;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug, Clone)]
#[clap(version = "0.1.0", author = "Starcoin Core Dev <dev@starcoin.org>")]
pub struct Options {
    #[clap(long, default_value = "http://main.seed.starcoin.org")]
    /// starcoin node http rpc url
    node_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DataProof {
    address: AccountAddress,
    index: u64,
    proof: Vec<String>,
}

fn map_rpc_error(err: RpcError) -> anyhow::Error {
    anyhow::anyhow!(format!("{}", err))
}

fn get_index_proofs(address: AccountAddress) -> Result<(Vec<u8>, Vec<u8>)> {
    let merkle_data = include_str!("../../../contrib-contracts/src/genesis-nft-address.json");
    let merkle_data: serde_json::Value = serde_json::from_str(merkle_data)?;
    let proofs: Vec<DataProof> = serde_json::from_value(merkle_data["proofs"].clone())?;
    let mint_proof = proofs
        .iter()
        .find(|p| p.address == address)
        .ok_or_else(|| anyhow::anyhow!("No Starcoin Genesis NFT for this sender"))?;
    let index = MoveValue::U64(mint_proof.index).simple_serialize().unwrap();
    let proofs = MoveValue::Vector(
        mint_proof
            .proof
            .iter()
            .map(|p| {
                hex::decode(p.as_str().strip_prefix("0x").unwrap_or_else(|| p.as_str())).unwrap()
            })
            .map(MoveValue::vector_u8)
            .collect(),
    )
    .simple_serialize()
    .unwrap();
    Ok((index, proofs))
}

#[tokio::main]
async fn main() -> Result<()> {
    let options: Options = Options::parse();
    let node_url = options.node_url.clone();
    let channel: RpcChannel = jsonrpc_core_client::transports::http::connect(node_url.as_str())
        .await
        .map_err(map_rpc_error)?;
    let private_key: AccountPrivateKey = {
        let pass = rpassword::prompt_password_stdout("Please Input Private Key: ")?;
        AccountPrivateKey::from_encoded_string(pass.trim())?
    };
    let sender: AccountAddress = {
        let default_address = private_key.public_key().derived_address();
        let address = rpassword::prompt_password_stdout(&format!(
            "Please input account address(default {}): ",
            &default_address
        ))?;
        if address.trim().is_empty() {
            default_address
        } else {
            AccountAddress::from_str(address.as_str())?
        }
    };
    let chain_client = ChainClient::from(channel.clone());
    let state_client = StateClient::from(channel.clone());
    let txpool_client = TxPoolClient::from(channel.clone());
    let chain_id: u8 = chain_client.id().await.map_err(map_rpc_error)?.id;
    let account_sequence_number = {
        let ap = AccessPath::new(sender, DataPath::Resource(account_struct_tag()));
        let account_data: Option<Vec<u8>> = state_client.get(ap).await.map_err(map_rpc_error)?;
        account_data
            .map(|account_data| AccountResource::decode(&account_data))
            .transpose()?
            .map(|r| r.sequence_number())
            .unwrap_or_default()
    };
    let (index, proofs) = get_index_proofs(sender)?;
    let script_function = ScriptFunction::new(
        ModuleId::new(
            genesis_address(),
            Identifier::new("GenesisNFTScripts").unwrap(),
        ),
        Identifier::new("mint").unwrap(),
        vec![],
        vec![index, proofs],
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let txn = RawUserTransaction::new_script_function(
        sender,
        account_sequence_number,
        script_function,
        40000000,
        1,
        now + 60 * 60 * 12,
        ChainId::new(chain_id),
    );
    let signature = private_key.sign(&txn);
    let signed_txn = SignedUserTransaction::new(txn, signature);
    let signed_txn_hex = hex::encode(signed_txn.encode()?);
    let txn_hash: HashValue = txpool_client
        .submit_hex_transaction(signed_txn_hex)
        .await
        .map_err(map_rpc_error)?;
    let txn_info: TransactionInfoView = loop {
        let txn_info = chain_client
            .get_transaction_info(txn_hash)
            .await
            .map_err(map_rpc_error)?;
        match txn_info {
            None => {
                println!("wait txn to be mined, {}", txn_hash);
                tokio::time::delay_for(Duration::from_secs(5)).await;
                continue;
            }
            Some(txn_info) => {
                break txn_info;
            }
        }
    };
    if txn_info.status != TransactionStatusView::Executed {
        return Err(anyhow::anyhow!(format!("{:?}", txn_info)));
    } else {
        println!(
            "txn {} mined in block {}, id: {}, gas_usd: {}",
            txn_hash, txn_info.block_number.0, txn_info.block_hash, txn_info.gas_used.0,
        );
    }
    //TODO: Display the minted NFT info
    Ok(())
}
