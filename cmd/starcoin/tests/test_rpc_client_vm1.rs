use anyhow::Result;
use itertools::assert_equal;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, StructTag, TypeTag, CORE_CODE_ADDRESS};
use starcoin_config::{BuiltinNetworkID, ChainNetwork, NodeConfig};
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::{HashValue, ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use starcoin_logger::prelude::debug;
use starcoin_rpc_api::types::TransactionStatusView;
use starcoin_rpc_client::RpcClient;
use starcoin_transaction_builder::peer_to_peer_txn_sent_as_association;
use starcoin_types::account::{Account, DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_vm_types::account_config;
use starcoin_vm_types::account_config::stc_type_tag;
use starcoin_vm_types::transaction::authenticator::{AccountPrivateKey, AccountPublicKey};
use starcoin_vm_types::transaction::{
    DryRunTransaction, EntryFunction, RawUserTransaction, SignedUserTransaction, TransactionPayload,
};
use std::sync::Arc;
use std::time::Duration;

pub fn create_signed_txn_with_association_account_test(
    acc: &Account,
    payload: TransactionPayload,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> Result<SignedUserTransaction> {
    acc.sign_txn(RawUserTransaction::new(
        account_config::association_address(),
        sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        net.chain_id(),
        String::from("0x1::STC::STC"),
    ))
}

pub fn transfer_scripts_peer_to_peer_test(
    payee: AccountAddress,
    amount: u128,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
            ident_str!("TransferScripts").to_owned(),
        ),
        ident_str!("peer_to_peer_v2").to_owned(),
        vec![TypeTag::Struct(Box::new(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("STC").unwrap(),
            name: Identifier::new("STC").unwrap(),
            type_args: vec![],
        }))],
        vec![
            bcs_ext::to_bytes(&payee).unwrap(),
            bcs_ext::to_bytes(&amount).unwrap(),
        ],
    ))
}

#[test]
pub fn test_connect_vm1_rpc_server() -> Result<()> {
    // please set the ipc file from local address
    let ipc_file = "/Users/bobong/.starcoin/dev/starcoin.ipc";

    // Connect to remote
    let ipc_client = RpcClient::connect_ipc(ipc_file).expect("connect ipc fail.");
    let state_root = ipc_client.state_get_state_root()?;
    debug!("vm1 state_root: {}", state_root);
    assert_ne!(state_root, HashValue::zero());

    // Impor two accounts into remote node
    // {
    //   "account": "0xd1969148774e82f576597fc870687adc",
    //   "private_key": "0x2c91292c160610cf3df2c62c730ca0e54a60fbf6204677cbe7e54a4b7fcd6659"
    // }
    // {
    //     "account": "0x45601528550cce4d6881577e23a5eafc",
    //     "private_key": "0x1a1135ee05d0d50cca4410bdd5709a4839bc560d58de8c3b274ee3e3575b48e4"
    // }
    //
    let account1_private_key = AccountPrivateKey::from_encoded_string(
        "0x2c91292c160610cf3df2c62c730ca0e54a60fbf6204677cbe7e54a4b7fcd6659",
    )?;
    let account1_public_key = account1_private_key.public_key();
    let account1 = Account::with_keypair(account1_private_key, account1_public_key, None);

    let account2_private_key = AccountPrivateKey::from_encoded_string(
        "0x1a1135ee05d0d50cca4410bdd5709a4839bc560d58de8c3b274ee3e3575b48e4",
    )?;
    let account2_public_key = account2_private_key.public_key();
    let account2 = Account::with_keypair(account2_private_key, account2_public_key, None);

    // let account_address1 = AccountAddress::from_hex_literal("0xd1969148774e82f576597fc870687adc")?;
    // let account_address2 = AccountAddress::from_hex_literal("0x45601528550cce4d6881577e23a5eafc")?;

    ipc_client
        .account_import(
            account1.address().clone(),
            account1.private_key().clone().to_bytes().to_vec(),
            "".to_string(),
        )
        .ok();

    ipc_client
        .account_import(
            account2.address().clone(),
            account2.private_key().clone().to_bytes().to_vec(),
            "".to_string(),
        )
        .ok();

    assert_eq!(
        account1.auth_key().to_string(),
        "0x6fb11390600e537f77cfd0cfc8047081d1969148774e82f576597fc870687adc"
    );
    assert_eq!(
        account2.auth_key().to_string(),
        "0x04a82bf2f306b0dd31f20d9449adc9f445601528550cce4d6881577e23a5eafc"
    );

    let account1_info = ipc_client.account_get(account1.address().clone())?;
    assert_eq!(
        account1_info.unwrap().public_key.to_encoded_string()?,
        "0x29e7ee6c40be7f4fc8c2eff20e13b4217d00b127f0158770161705df3aef7009"
    );

    // Check account exists
    let ret_acc_opt = ipc_client.account_get(account1.address().clone())?;
    assert!(ret_acc_opt.is_some());
    let ret_acc = ret_acc_opt.unwrap().address.to_hex_literal();
    assert_eq!(ret_acc, account1.address().clone().to_hex_literal());

    // Do transfer and accept STC for account1 and account2 in remote console.
    // Make sure the account1 and account2 have STC resource, and the account1 have balance

    // Do transfer from account1 to account2
    let network = ChainNetwork::new_builtin(BuiltinNetworkID::Dev);
    let txn = create_signed_txn_with_association_account_test(
        &account1,
        transfer_scripts_peer_to_peer_test(account1.address().clone(), 10000000000),
        0,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        network.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        &network,
    )?;
    let dry_run_output_view = ipc_client.dry_run_raw(DryRunTransaction {
        raw_txn: txn.raw_txn().clone(),
        public_key: txn.authenticator().public_key(),
    })?;

    assert_eq!(
        TransactionStatusView::Executed,
        dry_run_output_view.txn_output.status
    );
    Ok(())
}
