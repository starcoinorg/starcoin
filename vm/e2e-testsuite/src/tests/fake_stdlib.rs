// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::Sample;
use move_core_types::{
    account_address::AccountAddress, language_storage::TypeTag,
    transaction_argument::TransactionArgument, value::MoveValue,
};
use move_ir_compiler::Compiler;
use serde::Serialize;
use starcoin_language_e2e_tests::compile::compile_script;
use starcoin_vm_types::transaction::{Script, SignedUserTransaction, TransactionPayload};

fn to_vec(arg: TransactionArgument) -> Vec<u8> {
    MoveValue::from(arg)
        .simple_serialize()
        .expect("Arguments must serialize")
}

pub fn encode_peer_to_peer_with_metadata_script(
    currency: TypeTag,
    payee: AccountAddress,
    amount: u64,
    metadata: Vec<u8>,
    metadata_signature: Vec<u8>,
) -> Script {
    let compiler = compile_script(
        r#"
        script {
            use 0x1::TransferScripts;
            use 0x1::STC::STC;
            fun main(account: signer, payee: address, payee_auth_key: vector<u8>, amount: u128, metadata: vector<u8>) {
                TransferScripts::peer_to_peer_with_metadata<STC>(account, payee, payee_auth_key, amount, metadata);
        }
    }
    "#,
    );
    Script::new(
        compiler.unwrap(),
        vec![currency],
        vec![
            to_vec(TransactionArgument::Address(payee)),
            to_vec(TransactionArgument::U64(amount)),
            to_vec(TransactionArgument::U8Vector(metadata)),
            to_vec(TransactionArgument::U8Vector(metadata_signature)),
        ],
    )
    //Script::sample()
}

pub fn encode_tiered_mint_script(
    _coin_type: TypeTag,
    _sliding_nonce: u64,
    _designated_dealer_address: AccountAddress,
    _mint_amount: u64,
    _tier_index: u64,
) -> Script {
    // Script::new(
    //     TIERED_MINT_CODE.to_vec(),
    //     vec![coin_type],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(designated_dealer_address),
    //         TransactionArgument::U64(mint_amount),
    //         TransactionArgument::U64(tier_index),
    //     ],
    // )
    Script::sample()
}

pub fn encode_set_validator_operator_with_nonce_admin_script(
    _sliding_nonce: u64,
    _operator_name: Vec<u8>,
    _operator_account: AccountAddress,
) -> Script {
    // Script::new(
    //     SET_VALIDATOR_OPERATOR_WITH_NONCE_ADMIN_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::U8Vector(operator_name),
    //         TransactionArgument::Address(operator_account),
    //     ],
    // )
    Script::sample()
}

pub fn encode_set_validator_config_and_reconfigure_script(
    _validator_account: AccountAddress,
    _consensus_pubkey: Vec<u8>,
    _validator_network_addresses: Vec<u8>,
    _fullnode_network_addresses: Vec<u8>,
) -> Script {
    // Script::new(
    //     SET_VALIDATOR_CONFIG_AND_RECONFIGURE_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::Address(validator_account),
    //         TransactionArgument::U8Vector(consensus_pubkey),
    //         TransactionArgument::U8Vector(validator_network_addresses),
    //         TransactionArgument::U8Vector(fullnode_network_addresses),
    //     ],
    // )
    Script::sample()
}

pub fn encode_remove_validators_payload(_validators: Vec<AccountAddress>) -> SignedUserTransaction {
    // assert!(!validators.is_empty(), "Unexpected validator set length");
    // let mut script = template_path();
    // script.push("remove_validators.move");
    //
    // let script = {
    //     let mut hb = Handlebars::new();
    //     hb.set_strict_mode(true);
    //     hb.register_template_file("script", script).unwrap();
    //     let mut data = HashMap::new();
    //     data.insert("addresses", validators);
    //
    //     let output = hb.render("script", &data).unwrap();
    //
    //     compile_admin_script(output.as_str()).unwrap()
    // };
    //
    // WriteSetPayload::Script {
    //     script,
    //     execute_as: diem_root_address(),
    // }
    SignedUserTransaction::sample()
}

pub fn encode_custom_script<T: Serialize>(
    _script_name_in_templates: &str,
    _args: &T,
    _execute_as: Option<AccountAddress>,
) -> SignedUserTransaction {
    // let mut script = template_path();
    // script.push(script_name_in_templates);
    //
    // let script = {
    //     let mut hb = Handlebars::new();
    //     hb.register_template_file("script", script).unwrap();
    //     hb.set_strict_mode(true);
    //     let output = hb.render("script", args).unwrap();
    //
    //     compile_admin_script(output.as_str()).unwrap()
    // };
    //
    // WriteSetPayload::Script {
    //     script,
    //     execute_as: execute_as.unwrap_or_else(diem_root_address),
    // }
    SignedUserTransaction::sample()
}

pub fn encode_burn_txn_fees_script(_coin_type: TypeTag) -> Script {
    //Script::new(BURN_TXN_FEES_CODE.to_vec(), vec![coin_type], vec![])
    Script::sample()
}

pub fn encode_update_exchange_rate_script(
    _currency: TypeTag,
    _sliding_nonce: u64,
    _new_exchange_rate_numerator: u64,
    _new_exchange_rate_denominator: u64,
) -> Script {
    // Script::new(
    //     UPDATE_EXCHANGE_RATE_CODE.to_vec(),
    //     vec![currency],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::U64(new_exchange_rate_numerator),
    //         TransactionArgument::U64(new_exchange_rate_denominator),
    //     ],
    // )
    Script::sample()
}

pub fn encode_update_dual_attestation_limit_script(
    _sliding_nonce: u64,
    _new_micro_xdx_limit: u64,
) -> Script {
    // Script::new(
    //     UPDATE_DUAL_ATTESTATION_LIMIT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::U64(new_micro_xdx_limit),
    //     ],
    // )
    Script::sample()
}

pub fn build_fake_module_upgrade_plan() -> Script {
    Script::sample()
}
