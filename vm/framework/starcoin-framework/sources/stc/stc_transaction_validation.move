/// `starcoin transaction validation` manages:
/// 1. prologue and epilogue of transactions.
/// 2. prologue of blocks.
module starcoin_framework::stc_transaction_validation {

    use std::error;
    use std::signer;
    use starcoin_framework::easy_gas;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::stc_transaction_fee;
    use starcoin_framework::create_signer;
    use starcoin_std::math128;
    use starcoin_framework::stc_util;
    use starcoin_framework::coin;
    use starcoin_framework::account;

    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::stc_transaction_timeout;

    use starcoin_framework::chain_id;
    use starcoin_framework::system_addresses;

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
        // Can only be invoked by genesis account
        // assert!(
        //     signer::address_of(&account) == system_addresses::get_starcoin_framework(),
        //     error::requires_address(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST),
        // );
        system_addresses::assert_starcoin_framework(&account);

        // Check that the chain ID stored on-chain matches the chain ID
        // specified by the transaction
        assert!(chain_id::get() == chain_id, error::invalid_argument(EPROLOGUE_BAD_CHAIN_ID));

        let (stc_price, scaling_factor) = if (!stc_util::is_stc<TokenType>()) {
            // TODO(BobOng): [framework compatible] to debug easy gas oracle
            // (easy_gas::gas_oracle_read<TokenType>(), easy_gas::get_scaling_factor<TokenType>())
            (1, 1)
        } else {
            (1, 1)
        };

        txn_prologue<TokenType>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_authentication_key_preimage,
            txn_gas_price,
            txn_max_gas_units,
            stc_price,
            scaling_factor,
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
                transaction_publish_option::is_script_allowed(
                    signer::address_of(&account),
                ),
                error::invalid_argument(EPROLOGUE_SCRIPT_NOT_ALLOWED),
            );
        };
        // do nothing for TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION
    }


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
        system_addresses::assert_starcoin_framework(&account);

        let (stc_price, scaling_factor) = if (stc_util::is_stc<TokenType>()) {
            // TODO(BobOng): [framework compatible] to debug easy gas oracle
           // (easy_gas::gas_oracle_read<TokenType>(), easy_gas::get_scaling_factor<TokenType>())
            (1, 1)
        }else {
            (1, 1)
        };

        txn_epilogue<TokenType>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_authentication_key_preimage,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining,
            stc_price,
            scaling_factor
        );
        if (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE) {
            stc_transaction_package_validation::package_txn_epilogue(
                &account,
                txn_sender,
                txn_package_address,
                success,
            );
        }
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
    const EPROLOGUE_SIGNER_ALREADY_DELEGATED: u64 = 200;

    public fun txn_prologue<TokenType>(
        account: &signer,
        txn_sender: address,
        txn_sequence_number: u64,
        _txn_authentication_key_preimage: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        stc_price: u128,
        stc_price_scaling: u128
    ) {
        system_addresses::assert_starcoin_framework(account);

        // Verify that the transaction sender's account exists
        assert!(account::exists_at(txn_sender), error::not_found(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST));
        // Verify the account has not delegate its signer cap.
        assert!(
            !account::is_signer_capability_offered(txn_sender),
            error::invalid_state(EPROLOGUE_SIGNER_ALREADY_DELEGATED)
        );

        // TODO(BobOng): [framework upgrade] txn_authentication_key_preimage to be check
        // // Load the transaction sender's account
        // //let sender_account = borrow_global_mut<Account>(txn_sender);
        // if (account::is_dummy_auth_key_v2(txn_sender)) {
        //     // if sender's auth key is empty, use address as auth key for check transaction.
        //     assert!(
        //         Authenticator::derived_address(Hash::sha3_256(txn_authentication_key_preimage)) == txn_sender,
        //         error::invalid_argument(EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY)
        //     );
        // } else {
        //     // Check that the hash of the transaction's public key matches the account's auth key
        //     assert!(
        //         hash::sha3_256(txn_authentication_key_preimage) == account::authentication_key(txn_sender),
        //         error::invalid_argument(EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY)
        //     );
        // };

        // Check that the account has enough balance for all of the gas
        let (max_transaction_fee_stc, max_transaction_fee_token) =
            transaction_fee_simulate(
                txn_gas_price,
                txn_max_gas_units,
                0,
                stc_price,
                stc_price_scaling
            );

        assert!(
            max_transaction_fee_stc <= MAX_U64,
            error::invalid_argument(EPROLOGUE_CANT_PAY_GAS_DEPOSIT),
        );

        if (max_transaction_fee_stc > 0) {
            assert!(
                (txn_sequence_number as u128) < MAX_U64,
                error::out_of_range(EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG)
            );
            let balance_amount_token = coin::balance<TokenType>(txn_sender);
            assert!(
                balance_amount_token >= (max_transaction_fee_token as u64),
                error::invalid_argument(EPROLOGUE_CANT_PAY_GAS_DEPOSIT)
            );

            if (!stc_util::is_stc<TokenType>()) {
                let gas_fee_address = easy_gas::get_gas_fee_address();
                let balance_amount_stc = (coin::balance<STC>(gas_fee_address) as u128);
                assert!(
                    balance_amount_stc >= max_transaction_fee_stc,
                    error::invalid_argument(EPROLOGUE_CANT_PAY_GAS_DEPOSIT)
                );
            }
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

    /// The epilogue is invoked at the end of transactions.
    /// It collects gas and bumps the sequence number
    public fun txn_epilogue<TokenType>(
        account: &signer,
        txn_sender: address,
        _txn_sequence_number: u64,
        _txn_authentication_key_preimage: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        stc_price: u128,
        stc_price_scaling: u128,
    ) {
        system_addresses::assert_starcoin_framework(account);


        // Charge for gas
        let (transaction_fee_amount_stc, transaction_fee_amount_token) =
            transaction_fee_simulate(
                txn_gas_price,
                txn_max_gas_units,
                gas_units_remaining,
                stc_price,
                stc_price_scaling
            );
        assert!(
            coin::balance<TokenType>(txn_sender) >= (transaction_fee_amount_token as u64),
            error::out_of_range(EINSUFFICIENT_BALANCE)
        );

        if (!stc_util::is_stc<TokenType>()) {
            let gas_fee_address = easy_gas::get_gas_fee_address();
            let genesis_balance_amount_stc = (coin::balance<STC>(gas_fee_address) as u128);
            assert!(genesis_balance_amount_stc >= transaction_fee_amount_stc,
                error::invalid_argument(EPROLOGUE_CANT_PAY_GAS_DEPOSIT)
            );
        };

        // Bump the sequence number
        account::increment_sequence_number(txn_sender);

        // TODO(BobOng): [framework upgrade] txn_authentication_key_preimage to be check
        // Set auth key when user send transaction first.
        // if (Account::is_dummy_auth_key_v2(txn_sender) && !Vector::is_empty(&txn_authentication_key_preimage)) {
        //     Account::set_authentication_key(txn_sender, Hash::sha3_256(txn_authentication_key_preimage));
        // };

        if (transaction_fee_amount_stc <= 0) {
            return
        };

        let transaction_fee_token = coin::withdraw<TokenType>(
            &create_signer::create_signer(txn_sender),
            (transaction_fee_amount_token as u64)
        );

        if (!stc_util::is_stc<TokenType>()) {
            let gas_fee_address = easy_gas::get_gas_fee_address();
            coin::deposit<TokenType>(gas_fee_address, transaction_fee_token);

            let stc_fee_token = coin::withdraw<STC>(
                &create_signer::create_signer(gas_fee_address),
                (transaction_fee_amount_stc as u64)
            );
            stc_transaction_fee::pay_fee(stc_fee_token);
        } else {
            stc_transaction_fee::pay_fee(transaction_fee_token);
        };
    }

    /// Simulate the transaction fee
    ///
    public fun transaction_fee_simulate(
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        stc_price: u128,
        stc_price_scaling: u128,
    ): (u128, u128) {
        let transaction_fee_stc = (txn_gas_price * (txn_max_gas_units - gas_units_remaining) as u128);
        let transaction_fee_token = math128::mul_div(transaction_fee_stc, stc_price, stc_price_scaling);
        transaction_fee_token = if (transaction_fee_token == 0 && transaction_fee_stc > 0) {
            1
        } else {
            transaction_fee_token
        };
        (transaction_fee_stc, transaction_fee_token)
    }
}
