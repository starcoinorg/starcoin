use anyhow::Result;
use bcs_ext::BCSCodec;
use clap::Parser;
use jsonrpc_core_client::{RpcChannel, RpcError};
use serde::Deserialize;
use starcoin_crypto::{HashValue, ValidCryptoMaterialStringExt};
use starcoin_rpc_api::types::{TransactionInfoView, TransactionStatusView};
use starcoin_rpc_api::{
    chain::ChainClient, node::NodeClient, state::StateClient, txpool::TxPoolClient,
};
use starcoin_types::access_path::{AccessPath, DataPath};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::{
    account_struct_tag, genesis_address, stc_type_tag, AccountResource,
};
use starcoin_types::genesis_config::ChainId;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::authenticator::{AccountPrivateKey, AuthenticationKey};
use starcoin_types::transaction::{RawUserTransaction, ScriptFunction};
use starcoin_vm_types::transaction::SignedUserTransaction;
use starcoin_vm_types::value::MoveValue;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
#[derive(Parser, Debug, Clone)]
#[clap(version = "0.1.0", author = "Starcoin Core Dev <dev@starcoin.org>")]
pub struct Options {
    #[clap(long, default_value = "http://localhost:9850")]
    /// starcoin node http rpc url
    node_url: String,
    #[clap(short = 'i')]
    /// airdrop input csv. columns: `address,auth_key,amount`
    airdrop_file: PathBuf,
    #[clap(short, long, default_value = "32")]
    /// batch size to do transfer
    batch_size: usize,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct AirdropInfo {
    address: AccountAddress,
    auth_key: AuthenticationKey,
    amount: u128,
}

fn map_rpc_error(err: RpcError) -> anyhow::Error {
    anyhow::anyhow!(format!("{}", err))
}

#[tokio::main]
async fn main() -> Result<()> {
    let options: Options = Options::parse();
    let node_url = options.node_url.clone();
    let airdrop_file = options.airdrop_file.clone();
    let batch_size = options.batch_size;
    let channel: RpcChannel = jsonrpc_core_client::transports::http::connect(node_url.as_str())
        .await
        .map_err(map_rpc_error)?;
    let chain_client = ChainClient::from(channel.clone());
    let txpool_client = TxPoolClient::from(channel.clone());
    let state_client = StateClient::from(channel.clone());
    let node_client = NodeClient::from(channel.clone());
    let chain_id: u8 = chain_client.id().await.map_err(map_rpc_error)?.id;

    let airdrop_infos: Vec<AirdropInfo> = {
        let mut csv_reader = csv::ReaderBuilder::default()
            .has_headers(false)
            .from_path(airdrop_file.as_path())?;
        let mut leafs = Vec::with_capacity(4096);
        for record in csv_reader.deserialize() {
            let data = record?;
            leafs.push(data);
        }
        leafs
    };

    let private_key: AccountPrivateKey = {
        let pass = rpassword::prompt_password_stdout("Please Input Private Key: ")?;
        AccountPrivateKey::from_encoded_string(pass.trim())?
    };
    let sender: AccountAddress = {
        let default_address = private_key.public_key().derived_address();
        let address = rpassword::prompt_password_stdout(&format!(
            "Please Input Account Address(default {}): ",
            &default_address
        ))?;
        if address.trim().is_empty() {
            default_address
        } else {
            AccountAddress::from_str(address.as_str())?
        }
    };
    println!("Will act as sender {}", sender);

    // read from onchain
    let account_sequence_number = {
        let ap = AccessPath::new(sender, DataPath::Resource(account_struct_tag()));
        let account_data: Option<Vec<u8>> = state_client.get(ap).await.map_err(map_rpc_error)?;
        account_data
            .map(|account_data| AccountResource::decode(&account_data))
            .transpose()?
            .map(|r| r.sequence_number())
            .unwrap_or_default()
    };
    for (i, airdrops) in airdrop_infos.chunks(batch_size).into_iter().enumerate() {
        let addresses = MoveValue::Vector(
            airdrops
                .iter()
                .map(|info| info.address)
                .map(MoveValue::Address)
                .collect(),
        );
        let auth_keys = MoveValue::Vector(
            airdrops
                .iter()
                .map(|info| info.auth_key)
                .map(|v| MoveValue::vector_u8(v.to_vec()))
                .collect(),
        );
        let amounts = MoveValue::Vector(
            airdrops
                .iter()
                .map(|info| info.amount)
                .map(MoveValue::U128)
                .collect(),
        );

        let script_function = ScriptFunction::new(
            ModuleId::new(
                genesis_address(),
                Identifier::new("TransferScripts").unwrap(),
            ),
            Identifier::new("batch_peer_to_peer").unwrap(),
            vec![stc_type_tag()],
            vec![
                addresses.simple_serialize().unwrap(),
                auth_keys.simple_serialize().unwrap(),
                amounts.simple_serialize().unwrap(),
            ],
        );

        let now = node_client.info().await.map_err(map_rpc_error)?.now_seconds;
        let txn = RawUserTransaction::new_script_function(
            sender,
            account_sequence_number + i as u64,
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
                    // sleep 10s.
                    tokio::time::delay_for(Duration::from_secs(5)).await;
                    continue;
                }
                Some(txn_info) => {
                    break txn_info;
                }
            }
        };
        if txn_info.status != TransactionStatusView::Executed {
            eprintln!(
                "txn {:?} error: {:?}, please resume from user: {}",
                txn_hash,
                txn_info,
                airdrops.first().unwrap().address
            );
            break;
        } else {
            println!(
                "txn {} mined in block {}, id: {}, gas_usd: {}, airdrop users: {}-{}",
                txn_hash,
                txn_info.block_number.0,
                txn_info.block_hash,
                txn_info.gas_used.0,
                airdrops.first().unwrap().address,
                airdrops.last().unwrap().address
            );
        }
    }

    Ok(())
}
