// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tests::fake_stdlib::{
    self, encode_burn_txn_fees_script, encode_create_parent_vasp_account_script,
    encode_peer_to_peer_with_metadata_script,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    vm_status::KeptVMStatus,
};
use starcoin_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use starcoin_language_e2e_tests::{
    account::STC_TOKEN_CODE_STR, test_with_different_versions, versioning::CURRENT_RELEASE_VERSIONS,
};
use starcoin_vm_types::{
    account_config::{self, BurnEvent},
    transaction::authenticator::AuthenticationKey,
};

#[test]
fn burn_txn_fees() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        let sender = executor.create_raw_account();
        let dd = test_env.dd_account;
        let blessed = test_env.tc_account;

        executor.execute_and_apply(
            blessed
                .transaction()
                .script(encode_create_parent_vasp_account_script(
                    account_config::stc_type_tag(),
                    0,
                    *sender.address(),
                    sender.auth_key_prefix(),
                    vec![],
                    false,
                ))
                .sequence_number(test_env.tc_sequence_number)
                .sign(),
        );

        executor.execute_and_apply(
            dd.transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::stc_type_tag(),
                    *sender.address(),
                    10_000_000,
                    vec![],
                    vec![],
                ))
                .sequence_number(test_env.dd_sequence_number)
                .sign(),
        );

        let gas_used = {
            let privkey = Ed25519PrivateKey::generate_for_testing();
            let pubkey = privkey.public_key();
            let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
            //let args = vec![TransactionArgument::U8Vector(new_key_hash)];
            let status = executor.execute_and_apply(
                sender
                    .transaction()
                    .script(
                        // Script::new(
                        // LegacyStdlibScript::RotateAuthenticationKey
                        //     .compiled_bytes()
                        //     .into_vec(),
                        // vec![],
                        // args)
                        fake_stdlib::encode_rotate_authentication_key_script(new_key_hash)
                    )
                    .sequence_number(0)
                    .gas_unit_price(1)
                    .gas_currency_code(STC_TOKEN_CODE_STR)
                    .sign(),
            );
            assert_eq!(status.status().status(), Ok(KeptVMStatus::Executed));
            status.gas_used()
        };

        let xus_ty = TypeTag::Struct(Box::new(StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            module: Identifier::new("STC").unwrap(),
            name: Identifier::new("STC").unwrap(),
            type_params: vec![],
        }));

        let output = executor.execute_and_apply(
            blessed
                .transaction()
                .script(encode_burn_txn_fees_script(xus_ty))
                .sequence_number(test_env.tc_sequence_number.checked_add(1).unwrap())
                .sign(),
        );

        let burn_events: Vec<_> = output
            .events()
            .iter()
            .filter_map(|event| BurnEvent::try_from_bytes(event.event_data()).ok())
            .collect();

        assert_eq!(burn_events.len(), 1);
        assert!(burn_events
            .iter()
            .any(|event| event.token_code().to_string() == STC_TOKEN_CODE_STR));
        burn_events
            .iter()
            .for_each(|event| assert_eq!(event.amount(), gas_used as u128));
    }
    }
}
