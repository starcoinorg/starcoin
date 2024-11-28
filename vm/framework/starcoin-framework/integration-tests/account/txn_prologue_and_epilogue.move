//# init -n dev

//# faucet --addr alice --amount 10000000

//# faucet --addr Genesis


//# run --signers alice
// create txn sender account
script {
    use std::option;
    use starcoin_std::ed25519;
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;

    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(&option::destroy_some(valid_public_key));
        let address = account::auth_key_to_address(auth_key_vec);
        transfer_scripts::peer_to_peer_v2<STC>(account, address, 5000);
    }
}
// check: EXECUTED


//# run --signers alice
// prologue sender is not genesis
script {
    use std::option;
    use std::vector;
    use starcoin_framework::stc_transaction_validation;
    use starcoin_framework::coin;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_std::ed25519;

    fun test_prologue_sender_is_not_genesis(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(&option::destroy_some(valid_public_key));
        let txn_sender = account::auth_key_to_address(auth_key_vec);

        let seq = account::get_sequence_number(txn_sender);
        assert!(seq == 0, 1001);
        let balance = coin::balance<STC>(txn_sender);
        assert!(balance == 5000, 1001);

        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        stc_transaction_validation::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 327683"


//# run --signers Genesis
// gas is not enough
script {
    use std::option;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;
    use std::vector;
    use starcoin_std::ed25519;
    use starcoin_framework::stc_transaction_validation;

    fun test_gas_is_not_enough(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(&option::destroy_some(valid_public_key));
        let txn_sender = account::auth_key_to_address(auth_key_vec);

        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 10000; //EPROLOGUE_CANT_PAY_GAS_DEPOSIT

        stc_transaction_validation::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 1031"


//# run --signers Genesis
// invalid pub key
script {
    use std::option;
    use std::vector;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::stc_transaction_validation;
    use starcoin_std::ed25519;

    fun test_invalid_public_key(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(&option::destroy_some(valid_public_key));
        let txn_sender = account::auth_key_to_address(auth_key_vec);

        let wrong_txn_public_key = x"c48b687a";
        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        stc_transaction_validation::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            wrong_txn_public_key, //EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 263"


//# run --signers Genesis
// sequence number too new
script {
    use std::option;
    use std::vector;
    use starcoin_framework::stc_transaction_validation;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_std::ed25519;

    fun test_sequence_number_too_new(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(
                &option::destroy_some(valid_public_key)
            );
        let txn_sender = account::auth_key_to_address(auth_key_vec);

        let seq = account::get_sequence_number(txn_sender);
        assert!(seq == 0, 1005);

        let txn_sequence_number = 1; //EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        stc_transaction_validation::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 775"


//# run --signers Genesis
// successfully executed
script {
    use std::option;
    use std::vector;
    use starcoin_framework::stc_transaction_validation;
    use starcoin_std::ed25519;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;

    fun test_successfully_executed(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(
                &option::destroy_some(valid_public_key)
            );
        let txn_sender = account::auth_key_to_address(auth_key_vec);

        let seq = account::get_sequence_number(txn_sender);
        assert!(seq == 0, 1005);

        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        stc_transaction_validation::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );

        // execute the txn...

        let gas_units_remaining = 10;

        stc_transaction_validation::txn_epilogue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining,
        );
        let seq = account::get_sequence_number(txn_sender);
        assert!(seq == 1, 1006);
    }
}
// check: EXECUTED


//# run --signers Genesis
// sequence number too old
script {
    use std::option;
    use std::vector;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::stc_transaction_validation;
    use starcoin_std::ed25519;

    fun test_sequence_number_too_old(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(
                &option::destroy_some(valid_public_key)
            );
        let txn_sender = account::auth_key_to_address(auth_key_vec);

        let seq = account::get_sequence_number(txn_sender);
        assert!(seq == 1, 1005);

        let txn_sequence_number = 0; //EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        stc_transaction_validation::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 519"


//# run --signers Genesis
// epilouge insufficient balance
script {
    use std::option;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;
    use std::vector;
    use starcoin_framework::stc_transaction_validation;
    use starcoin_std::ed25519;

    fun test_epilogue_insufficient_balance(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(
                &option::destroy_some(valid_public_key)
            );
        let txn_sender = account::auth_key_to_address(auth_key_vec);
        let seq = account::get_sequence_number(txn_sender);
        assert!(seq == 1, 1007);

        let txn_sequence_number = 1;
        let txn_gas_price = 1;
        let txn_max_gas_units = 6000; //EINSUFFICIENT_BALANCE
        let gas_units_remaining = 10;

        stc_transaction_validation::txn_epilogue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining
        );
    }
}
// check: "Keep(ABORTED { code: 2568"


//# run --signers alice
// epilogue sender is not genesis
script {
    use std::option;
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;
    use std::vector;
    use starcoin_framework::stc_transaction_validation;
    use starcoin_std::ed25519;

    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let valid_public_key = ed25519::new_validated_public_key_from_bytes(txn_public_key);
        assert!(option::is_some(&valid_public_key), 1001);
        vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let auth_key_vec =
            ed25519::validated_public_key_to_authentication_key(
                &option::destroy_some(valid_public_key)
            );
        let txn_sender = account::auth_key_to_address(auth_key_vec);

        let seq = account::get_sequence_number(txn_sender);
        assert!(seq == 1, 1007);

        let txn_sequence_number = 1;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;
        let gas_units_remaining = 10;

        stc_transaction_validation::txn_epilogue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining
        );
    }
}
// check: "Keep(ABORTED { code: 2818"