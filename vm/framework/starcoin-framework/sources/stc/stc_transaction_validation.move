/// `starcoin transaction validation` manages:
/// 1. prologue and epilogue of transactions.
/// 2. prologue of blocks.
module starcoin_framework::stc_transaction_validation {

    use std::error;
    use std::hash;
    use std::signer;
    use std::vector;
    use starcoin_std::debug;

    use starcoin_framework::account;
    use starcoin_framework::chain_id;
    use starcoin_framework::coin;
    use starcoin_framework::create_signer;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::stc_transaction_fee;
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::stc_transaction_timeout;
    use starcoin_framework::stc_util;
    use starcoin_framework::system_addresses;
    use starcoin_framework::transaction_publish_option;

    const TXN_PAYLOAD_TYPE_SCRIPT: u8 = 0;
    const TXN_PAYLOAD_TYPE_PACKAGE: u8 = 1;
    const TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION: u8 = 2;

    const EPROLOGUE_ACCOUNT_DOES_NOT_EXIST: u64 = 0;
    const EPROLOGUE_TRANSACTION_EXPIRED: u64 = 5;
    const EPROLOGUE_BAD_CHAIN_ID: u64 = 6;
    const EPROLOGUE_MODULE_NOT_ALLOWED: u64 = 7;
    const EPROLOGUE_SCRIPT_NOT_ALLOWED: u64 = 8;


    /// The prologue is invoked at the beginning of every transaction
    /// It verifies:
    /// - The account's auth key matches the transaction's public key
    /// - That the account has enough balance to pay for all of the gas
    /// - That the sequence number matches the transaction's sequence key
    public fun prologue<TokenType>(
        account: signer,
        txn_sender: address,
        txn_sequence_number: u64,
        txn_authentication_key_preimage: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        txn_payload_type: u8,
        txn_script_or_package_hash: vector<u8>,
        txn_package_address: address,
    ) {
        debug::print(&std::string::utf8(b"transaction_validation::prologue | Entered"));

        // Can only be invoked by genesis account
        // assert!(
        //     signer::address_of(&account) == system_addresses::get_starcoin_framework(),
        //     error::requires_address(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST),
        // );
        system_addresses::assert_starcoin_framework(&account);

        // Check that the chain ID stored on-chain matches the chain ID
        // specified by the transaction
        assert!(chain_id::get() == chain_id, error::invalid_argument(EPROLOGUE_BAD_CHAIN_ID));

        txn_prologue<TokenType>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_authentication_key_preimage,
            txn_gas_price,
            txn_max_gas_units,
        );

        assert!(
            stc_transaction_timeout::is_valid_transaction_timestamp(txn_expiration_time),
            error::invalid_argument(EPROLOGUE_TRANSACTION_EXPIRED),
        );

        if (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE) {
            // stdlib upgrade is not affected by PublishOption
            if (txn_package_address != system_addresses::get_starcoin_framework()) {
                assert!(
                    transaction_publish_option::is_module_allowed(signer::address_of(&account)),
                    error::invalid_argument(EPROLOGUE_MODULE_NOT_ALLOWED),
                );
            };
            stc_transaction_package_validation::package_txn_prologue_v2(
                &account,
                txn_sender,
                txn_package_address,
                txn_script_or_package_hash,
            );
        } else if (txn_payload_type == TXN_PAYLOAD_TYPE_SCRIPT) {
            assert!(
                transaction_publish_option::is_script_allowed(signer::address_of(&account), ),
                error::invalid_argument(EPROLOGUE_SCRIPT_NOT_ALLOWED),
            );
        };
        debug::print(&std::string::utf8(b"transaction_validation::prologue | Exited"));
        // do nothing for TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION
    }

    /// Migration from old StarcoinFramework TransactionManager::epilogue
    /// The epilogue is invoked at the end of transactions.
    /// It collects gas and bumps the sequence number
    public fun epilogue<TokenType>(
        account: signer,
        txn_sender: address,
        txn_sequence_number: u64,
        txn_authentication_key_preimage: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        txn_payload_type: u8,
        _txn_script_or_package_hash: vector<u8>,
        txn_package_address: address,
        // txn execute success or fail.
        success: bool,
    ) {
        debug::print(&std::string::utf8(b"transaction_validation::epilogue | Entered"));

        system_addresses::assert_starcoin_framework(&account);
        txn_epilogue<TokenType>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_authentication_key_preimage,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining,
        );
        if (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE) {
            stc_transaction_package_validation::package_txn_epilogue(
                &account,
                txn_sender,
                txn_package_address,
                success,
            );
        };

        debug::print(&std::string::utf8(b"transaction_validation::epilogue | Exited"));
    }

    const MAX_U64: u128 = 18446744073709551615;
    const EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 2;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 3;
    const EPROLOGUE_CANT_PAY_GAS_DEPOSIT: u64 = 4;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG: u64 = 9;
    const EINSUFFICIENT_BALANCE: u64 = 10;
    const ECOIN_DEPOSIT_IS_ZERO: u64 = 15;
    const EDEPRECATED_FUNCTION: u64 = 19;
    const EBAD_TRANSACTION_FEE_TOKEN: u64 = 18;
    const EPROLOGUE_SIGNER_ALREADY_DELEGATED: u64 = 200;

    /// Migration from old StarcoinFramework Account::txn_prologue
    public fun txn_prologue<TokenType>(
        account: &signer,
        txn_sender: address,
        txn_sequence_number: u64,
        txn_authentication_key_preimage: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
    ) {
        system_addresses::assert_starcoin_framework(account);

        // Verify that the transaction sender's account exists
        assert!(account::exists_at(txn_sender), error::not_found(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST));
        // Verify the account has not delegate its signer cap.
        assert!(
            !account::is_signer_capability_offered(txn_sender),
            error::invalid_state(EPROLOGUE_SIGNER_ALREADY_DELEGATED)
        );

        // txn_authentication_key_preimage to be check
        // Load the transaction sender's account
        if (account::is_account_zero_auth_key(txn_sender)) {
            // if sender's auth key is empty, use address as auth key for check transaction.
            assert!(
                account::auth_key_to_address(hash::sha3_256(txn_authentication_key_preimage)) == txn_sender,
                error::invalid_argument(EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY)
            );
        } else {
            // Check that the hash of the transaction's public key matches the account's auth key
            assert!(
                //hash::sha3_256(txn_authentication_key_preimage) == *&sender_account.authentication_key,
                account::is_account_auth_key(txn_sender, hash::sha3_256(txn_authentication_key_preimage)),
                error::invalid_argument(EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY)
            );
        };


        assert!(
            (txn_gas_price as u128) * (txn_max_gas_units as u128) <= MAX_U64,
            error::invalid_argument(EPROLOGUE_CANT_PAY_GAS_DEPOSIT),
        );

        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        if (max_transaction_fee > 0) {
            assert!(
                stc_util::is_stc<TokenType>(),
                error::invalid_argument(EBAD_TRANSACTION_FEE_TOKEN)
            );

            let balance_amount = coin::balance<TokenType>(txn_sender);
            assert!(balance_amount >= max_transaction_fee, error::invalid_argument(EPROLOGUE_CANT_PAY_GAS_DEPOSIT));

            assert!(
                (txn_sequence_number as u128) < MAX_U64,
                error::out_of_range(EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG)
            );
        };
        // Check that the transaction sequence number matches the sequence number of the account
        assert!(
            txn_sequence_number >= account::get_sequence_number(txn_sender),
            error::invalid_argument(EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD)
        );
        assert!(
            txn_sequence_number == account::get_sequence_number(txn_sender),
            error::invalid_argument(EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW)
        );
    }

    /// Migration from old StarcoinFramework Account::txn_eiplogue
    /// The epilogue is invoked at the end of transactions.
    /// It collects gas and bumps the sequence number
    public fun txn_epilogue<TokenType>(
        account: &signer,
        txn_sender: address,
        _txn_sequence_number: u64,
        txn_authentication_key_preimage: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
    ) {
        system_addresses::assert_starcoin_framework(account);


        // Charge for gas
        let transaction_fee_amount = (txn_gas_price * (txn_max_gas_units - gas_units_remaining) as u128);
        assert!(
            coin::balance<STC>(txn_sender) >= (transaction_fee_amount as u64),
            error::out_of_range(EINSUFFICIENT_BALANCE)
        );

        // Bump the sequence number
        account::increment_sequence_number(txn_sender);

        // Set auth key when user send transaction first.
        if (account::is_account_zero_auth_key(txn_sender) &&
            !vector::is_empty(&txn_authentication_key_preimage)) {
            account::rotate_authentication_key_internal(
                &create_signer::create_signer(txn_sender),
                hash::sha3_256(txn_authentication_key_preimage)
            )
        };

        if (transaction_fee_amount > 0) {
            let transaction_fee = coin::withdraw<STC>(
                &create_signer::create_signer(txn_sender),
                (transaction_fee_amount as u64)
            );
            stc_transaction_fee::pay_fee(transaction_fee);
        };
    }
}
