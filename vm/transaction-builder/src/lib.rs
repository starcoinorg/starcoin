// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config;
use starcoin_vm_types::account_config::{
    association_address, config_address, mint_address, stc_type_tag, transaction_fee_address,
};
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::transaction::helpers::TransactionSigner;
use starcoin_vm_types::transaction::{
    Module, RawUserTransaction, Script, SignedUserTransaction, Transaction, TransactionArgument,
    TransactionPayload, UpgradePackage,
};
use std::time::Duration;

use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use stdlib::init_scripts::InitScript;
pub use stdlib::transaction_scripts::{CompiledBytes, StdlibScript};
pub use stdlib::{stdlib_modules, StdLibOptions};

pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const TXN_RESERVED: u64 = 2_000_000;

pub fn build_transfer_from_association(
    addr: AccountAddress,
    auth_key_prefix: Vec<u8>,
    association_sequence_num: u64,
    amount: u64,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        auth_key_prefix,
        association_sequence_num,
        amount,
    ))
}

pub fn build_transfer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
    gas_price: u64,
    max_gas: u64,
) -> RawUserTransaction {
    build_transfer_txn_by_coin_type(
        sender,
        receiver,
        receiver_auth_key_prefix,
        seq_num,
        amount,
        gas_price,
        max_gas,
        stc_type_tag(),
    )
}

pub fn build_transfer_txn_by_coin_type(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    raw_peer_to_peer_txn(
        sender,
        receiver,
        receiver_auth_key_prefix,
        amount,
        seq_num,
        gas_price,
        max_gas,
        coin_type,
    )
}

pub fn build_accept_coin_txn(
    sender: AccountAddress,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    raw_accept_coin_txn(sender, seq_num, gas_price, max_gas, coin_type)
}

pub fn raw_peer_to_peer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    transfer_amount: u64,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(receiver));
    args.push(TransactionArgument::U8Vector(receiver_auth_key_prefix));
    args.push(TransactionArgument::U64(transfer_amount));

    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(Script::new(
            StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
            vec![coin_type],
            args,
        )),
        max_gas,
        gas_price,
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
}

pub fn raw_accept_coin_txn(
    sender: AccountAddress,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(Script::new(
            StdlibScript::AcceptCoin.compiled_bytes().into_vec(),
            vec![coin_type],
            vec![],
        )),
        max_gas,
        gas_price,
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
}

pub fn encode_create_account_script(
    account_address: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    initial_balance: u64,
) -> Script {
    Script::new(
        StdlibScript::CreateAccount.compiled_bytes().into_vec(),
        vec![],
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(initial_balance),
        ],
    )
}

pub fn encode_transfer_script(
    recipient: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    Script::new(
        StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
        vec![stc_type_tag()],
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
        ],
    )
}

pub fn peer_to_peer_txn_sent_as_association(
    recipient: AccountAddress,
    auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
) -> SignedUserTransaction {
    crate::create_signed_txn_with_association_account(
        TransactionPayload::Script(encode_transfer_script(&recipient, auth_key_prefix, amount)),
        seq_num,
        TXN_RESERVED,
        1,
    )
}

//this only work for DEV,
pub fn create_signed_txn_with_association_account(
    payload: TransactionPayload,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
) -> SignedUserTransaction {
    ChainNetwork::Dev
        .get_config()
        .pre_mine_config
        .as_ref()
        .expect("Dev network pre mine config should exist")
        .sign_txn(RawUserTransaction::new(
            account_config::association_address(),
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            // TTL is 86400s. Initial time was set to 0.
            Duration::from_secs(DEFAULT_EXPIRATION_TIME),
        ))
        .expect("Sign txn should work.")
}

pub fn build_upgrade_package(
    net: ChainNetwork,
    stdlib_option: StdLibOptions,
    with_init_script: bool,
) -> Result<UpgradePackage> {
    let modules = stdlib_modules(stdlib_option);
    let mut package = UpgradePackage::new_with_modules(
        modules
            .iter()
            .map(|m| {
                let mut blob = vec![];
                m.serialize(&mut blob)
                    .expect("serializing stdlib must work");
                Module::new(blob)
            })
            .collect(),
    );
    if with_init_script {
        let chain_config = net.get_config();

        let genesis_auth_key = chain_config
            .pre_mine_config
            .as_ref()
            .map(|pre_mine_config| AuthenticationKey::ed25519(&pre_mine_config.public_key).to_vec())
            .unwrap_or_else(|| vec![0u8; AuthenticationKey::LENGTH]);

        package.add_script(
            Some(association_address()),
            Script::new(
                InitScript::AssociationInit.compiled_bytes().into_vec(),
                vec![],
                vec![TransactionArgument::U8Vector(genesis_auth_key)],
            ),
        );

        let publish_option_bytes = scs::to_bytes(&chain_config.vm_config.publishing_option)
            .expect("Cannot serialize publishing option");
        let instruction_schedule =
            scs::to_bytes(&chain_config.vm_config.gas_schedule.instruction_table)
                .expect("Cannot serialize gas schedule");
        let native_schedule = scs::to_bytes(&chain_config.vm_config.gas_schedule.native_table)
            .expect("Cannot serialize gas schedule");
        package.add_script(
            Some(config_address()),
            Script::new(
                InitScript::ConfigInit.compiled_bytes().into_vec(),
                vec![],
                vec![
                    TransactionArgument::U8Vector(publish_option_bytes),
                    TransactionArgument::U8Vector(instruction_schedule),
                    TransactionArgument::U8Vector(native_schedule),
                    TransactionArgument::U64(chain_config.reward_halving_interval),
                    TransactionArgument::U64(chain_config.base_block_reward),
                    TransactionArgument::U64(chain_config.reward_delay),
                ],
            ),
        );

        package.add_script(
            Some(association_address()),
            Script::new(
                InitScript::STCInit.compiled_bytes().into_vec(),
                vec![],
                vec![],
            ),
        );

        package.add_script(
            Some(mint_address()),
            Script::new(
                InitScript::MintInit.compiled_bytes().into_vec(),
                vec![],
                vec![],
            ),
        );
        let pre_mine_percent = chain_config
            .pre_mine_config
            .as_ref()
            .map(|cfg| cfg.pre_mine_percent)
            .unwrap_or(0);
        package.add_script(
            Some(association_address()),
            Script::new(
                InitScript::PreMineInit.compiled_bytes().into_vec(),
                vec![],
                vec![
                    TransactionArgument::U64(chain_config.total_supply),
                    TransactionArgument::U64(pre_mine_percent),
                ],
            ),
        );

        package.add_script(
            Some(transaction_fee_address()),
            Script::new(
                InitScript::FeeInit.compiled_bytes().into_vec(),
                vec![],
                vec![],
            ),
        );
    }
    Ok(package)
}
