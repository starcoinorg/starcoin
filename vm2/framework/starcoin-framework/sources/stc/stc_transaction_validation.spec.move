spec starcoin_framework::stc_transaction_validation {
    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    spec prologue {
        use std::hash;
        use starcoin_framework::stc_block;

        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if !exists<chain_id::ChainId>(system_addresses::get_starcoin_framework());
        aborts_if chain_id::get() != chain_id;
        aborts_if !exists<account::Account>(txn_sender);
        aborts_if hash::sha3_256(txn_authentication_key_preimage) != global<account::Account>(
            txn_sender
        ).authentication_key;
        aborts_if txn_gas_price * txn_max_gas_units > max_u64();

        // include timestamp::AbortsIfTimestampNotExists;
        include stc_block::AbortsIfBlockMetadataNotExist;

        aborts_if txn_gas_price * txn_max_gas_units > 0 && !exists<coin::CoinStore<TokenType>>(txn_sender);
        aborts_if txn_gas_price * txn_max_gas_units > 0 && txn_sequence_number >= max_u64();
        aborts_if txn_sequence_number < global<account::Account>(txn_sender).sequence_number;
        aborts_if txn_sequence_number != global<account::Account>(txn_sender).sequence_number;
        include stc_transaction_timeout::AbortsIfTimestampNotValid;
        aborts_if !stc_transaction_timeout::spec_is_valid_transaction_timestamp(txn_expiration_time);
        include transaction_publish_option::AbortsIfTxnPublishOptionNotExistWithBool {
            is_script_or_package: (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE || txn_payload_type == TXN_PAYLOAD_TYPE_SCRIPT),
        };

        aborts_if txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE
            && txn_package_address != system_addresses::get_starcoin_framework()
            && !transaction_publish_option::spec_is_module_allowed(signer::address_of(account));
        aborts_if txn_payload_type == TXN_PAYLOAD_TYPE_SCRIPT
            && !transaction_publish_option::spec_is_script_allowed(signer::address_of(account));

        include stc_transaction_package_validation::CheckPackageTxnAbortsIfWithType {
            is_package: (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE),
            sender: txn_sender,
            package_address: txn_package_address,
            package_hash: txn_script_or_package_hash
        };
    }

    spec epilogue {
        pragma verify = false;//fixme : timeout
        // include CoreAddresses::AbortsIfNotGenesisAddress;
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if !exists<account::Account>(txn_sender);
        aborts_if !exists<coin::CoinStore<TokenType>>(txn_sender);
        aborts_if txn_max_gas_units < gas_units_remaining;
        aborts_if txn_sequence_number + 1 > max_u64();
        aborts_if txn_gas_price * (txn_max_gas_units - gas_units_remaining) > max_u64();
        include stc_transaction_package_validation::AbortsIfPackageTxnEpilogue {
            is_package: (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE),
            package_address: txn_package_address,
            success,
        };
    }

    spec txn_epilogue {
        use starcoin_framework::coin;
        use starcoin_framework::account;

        pragma verify = false; // Todo: fix me, cost too much time
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if !exists<account::Account>(txn_sender);
        aborts_if !exists<coin::CoinStore<TokenType>>(txn_sender);
        aborts_if _txn_sequence_number + 1 > max_u64();
        aborts_if txn_max_gas_units < gas_units_remaining;
    }
}