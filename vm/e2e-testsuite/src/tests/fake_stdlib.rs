// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::Sample;
use move_core_types::{account_address::AccountAddress, language_storage::TypeTag};
use serde::Serialize;
use starcoin_vm_types::transaction::{Script, SignedUserTransaction, TransactionPayload};

pub fn encode_create_validator_account_script(
    _sliding_nonce: u64,
    _new_account_address: AccountAddress,
    _auth_key_prefix: Vec<u8>,
    _human_name: Vec<u8>,
) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_create_validator_operator_account_script(
    _sliding_nonce: u64,
    _new_account_address: AccountAddress,
    _auth_key_prefix: Vec<u8>,
    _human_name: Vec<u8>,
) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_set_validator_operator_script(
    _operator_name: Vec<u8>,
    _operator_account: AccountAddress,
) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_register_validator_config_script(
    _validator_account: AccountAddress,
    _consensus_pubkey: Vec<u8>,
    _validator_network_addresses: Vec<u8>,
    _fullnode_network_addresses: Vec<u8>,
) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_add_validator_and_reconfigure_script(
    _sliding_nonce: u64,
    _validator_name: Vec<u8>,
    _validator_address: AccountAddress,
) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_preburn_script(_token: TypeTag, _amount: u64) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_burn_script(
    _token: TypeTag,
    _sliding_nonce: u64,
    _preburn_address: AccountAddress,
) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_cancel_burn_script(_token: TypeTag, _preburn_address: AccountAddress) -> Script {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    Script::sample()
}

pub fn encode_burn_with_amount_script_function(
    _token: TypeTag,
    _sliding_nonce: u64,
    _preburn_address: AccountAddress,
    _amount: u64,
) -> TransactionPayload {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    TransactionPayload::Script(Script::sample())
}

pub fn encode_cancel_burn_with_amount_script_function(
    _token: TypeTag,
    _preburn_address: AccountAddress,
    _amount: u64,
) -> TransactionPayload {
    // Script::new(
    //     CREATE_VALIDATOR_OPERATOR_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //     ],
    // )
    TransactionPayload::Script(Script::sample())
}

pub fn encode_rotate_authentication_key_with_nonce_admin_script(
    _sliding_nonce: u64,
    _new_key: Vec<u8>,
) -> Script {
    Script::sample()
}
// pub fn encode_rotate_authentication_key_with_nonce_admin_script_function(
//     _sliding_nonce: u64,
//     _new_key: Vec<u8>,
// ) -> TransactionPayload {
//     TransactionPayload::Script(Script::sample())
// }

pub fn encode_create_parent_vasp_account_script(
    _coin_type: TypeTag,
    _sliding_nonce: u64,
    _new_account_address: AccountAddress,
    _auth_key_prefix: Vec<u8>,
    _human_name: Vec<u8>,
    _add_all_currencies: bool,
) -> Script {
    // Script::new(
    //     CREATE_PARENT_VASP_ACCOUNT_CODE.to_vec(),
    //     vec![coin_type],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(new_account_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //         TransactionArgument::Bool(add_all_currencies),
    //     ],
    // )
    Script::sample()
}

pub fn encode_freeze_account_script(
    sliding_nonce: u64,
    to_freeze_account: AccountAddress,
) -> Script {
    // Script::new(
    //     FREEZE_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(to_freeze_account),
    //     ],
    // )
    Script::sample()
}

pub fn encode_unfreeze_account_script(
    _sliding_nonce: u64,
    _to_unfreeze_account: AccountAddress,
) -> Script {
    // Script::new(
    //     UNFREEZE_ACCOUNT_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(to_unfreeze_account),
    //     ],
    // )
    Script::sample()
}

pub fn encode_rotate_dual_attestation_info_script(_new_url: Vec<u8>, _new_key: Vec<u8>) -> Script {
    // Script::new(
    //     ROTATE_DUAL_ATTESTATION_INFO_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::U8Vector(new_url),
    //         TransactionArgument::U8Vector(new_key),
    //     ],
    // )
    Script::sample()
}

pub fn encode_create_child_vasp_account_script(
    _coin_type: TypeTag,
    _child_address: AccountAddress,
    _auth_key_prefix: Vec<u8>,
    _add_all_currencies: bool,
    _child_initial_balance: u64,
) -> Script {
    // Script::new(
    //     CREATE_CHILD_VASP_ACCOUNT_CODE.to_vec(),
    //     vec![coin_type],
    //     vec![
    //         TransactionArgument::Address(child_address),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::Bool(add_all_currencies),
    //         TransactionArgument::U64(child_initial_balance),
    //     ],
    // )
    Script::sample()
}

pub fn encode_create_child_vasp_account_script_function(
    _coin_type: TypeTag,
    _child_address: AccountAddress,
    _auth_key_prefix: Vec<u8>,
    _add_all_currencies: bool,
    _child_initial_balance: u64,
) -> TransactionPayload {
    // TransactionPayload::ScriptFunction(ScriptFunction::new(
    //     ModuleId::new(
    //         AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
    //         ident_str!("AccountCreationScripts").to_owned(),
    //     ),
    //     ident_str!("create_child_vasp_account").to_owned(),
    //     vec![coin_type],
    //     vec![
    //         bcs::to_bytes(&child_address).unwrap(),
    //         bcs::to_bytes(&auth_key_prefix).unwrap(),
    //         bcs::to_bytes(&add_all_currencies).unwrap(),
    //         bcs::to_bytes(&child_initial_balance).unwrap(),
    //     ],
    // ))
    TransactionPayload::Script(Script::sample())
}

pub fn encode_peer_to_peer_with_metadata_script(
    _currency: TypeTag,
    _payee: AccountAddress,
    _amount: u64,
    _metadata: Vec<u8>,
    _metadata_signature: Vec<u8>,
) -> Script {
    // Script::new(
    //     PEER_TO_PEER_WITH_METADATA_CODE.to_vec(),
    //     vec![currency],
    //     vec![
    //         TransactionArgument::Address(payee),
    //         TransactionArgument::U64(amount),
    //         TransactionArgument::U8Vector(metadata),
    //         TransactionArgument::U8Vector(metadata_signature),
    //     ],
    // )
    Script::sample()
}

pub fn encode_create_designated_dealer_script(
    _currency: TypeTag,
    _sliding_nonce: u64,
    _addr: AccountAddress,
    _auth_key_prefix: Vec<u8>,
    _human_name: Vec<u8>,
    _add_all_currencies: bool,
) -> Script {
    // Script::new(
    //     CREATE_DESIGNATED_DEALER_CODE.to_vec(),
    //     vec![currency],
    //     vec![
    //         TransactionArgument::U64(sliding_nonce),
    //         TransactionArgument::Address(addr),
    //         TransactionArgument::U8Vector(auth_key_prefix),
    //         TransactionArgument::U8Vector(human_name),
    //         TransactionArgument::Bool(add_all_currencies),
    //     ],
    // )
    Script::sample()
}

pub fn encode_publish_shared_ed25519_public_key_script(_public_key: Vec<u8>) -> Script {
    // Script::new(
    //     PUBLISH_SHARED_ED25519_PUBLIC_KEY_CODE.to_vec(),
    //     vec![],
    //     vec![TransactionArgument::U8Vector(public_key)],
    // )
    Script::sample()
}

pub fn encode_rotate_shared_ed25519_public_key_script(_public_key: Vec<u8>) -> Script {
    // Script::new(
    //     ROTATE_SHARED_ED25519_PUBLIC_KEY_CODE.to_vec(),
    //     vec![],
    //     vec![TransactionArgument::U8Vector(public_key)],
    // )
    Script::sample()
}

pub fn encode_create_recovery_address_script() -> Script {
    //Script::new(CREATE_RECOVERY_ADDRESS_CODE.to_vec(), vec![], vec![])
    Script::sample()
}

pub fn encode_add_recovery_rotation_capability_script(_recovery_address: AccountAddress) -> Script {
    // Script::new(
    //     ADD_RECOVERY_ROTATION_CAPABILITY_CODE.to_vec(),
    //     vec![],
    //     vec![TransactionArgument::Address(recovery_address)],
    // )
    Script::sample()
}

pub fn encode_rotate_authentication_key_with_recovery_address_script(
    _recovery_address: AccountAddress,
    _to_recover: AccountAddress,
    _new_key: Vec<u8>,
) -> Script {
    // Script::new(
    //     ROTATE_AUTHENTICATION_KEY_WITH_RECOVERY_ADDRESS_CODE.to_vec(),
    //     vec![],
    //     vec![
    //         TransactionArgument::Address(recovery_address),
    //         TransactionArgument::Address(to_recover),
    //         TransactionArgument::U8Vector(new_key),
    //     ],
    // )
    Script::sample()
}

pub fn encode_tiered_mint_script(
    coin_type: TypeTag,
    sliding_nonce: u64,
    designated_dealer_address: AccountAddress,
    mint_amount: u64,
    tier_index: u64,
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

pub fn encode_halt_network_payload() -> SignedUserTransaction {
    // let mut script = template_path();
    // script.push("halt_transactions.move");
    //
    // WriteSetPayload::Script {
    //     script: Script::new(
    //         compile_script(script.to_str().unwrap().to_owned()),
    //         vec![],
    //         vec![],
    //     ),
    //     execute_as: diem_root_address(),
    // }
    SignedUserTransaction::sample()
}

pub fn encode_rotate_authentication_key_script(_new_key: Vec<u8>) -> Script {
    // Script::new(
    //     ROTATE_AUTHENTICATION_KEY_CODE.to_vec(),
    //     vec![],
    //     vec![TransactionArgument::U8Vector(new_key)],
    // )
    Script::sample()
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
