address 0x1 {
module TransactionManager {
    use 0x1::TransactionTimeout;
    use 0x1::Signer;
    use 0x1::Token::{Self, Token};
    use 0x1::CoreAddresses;
    use 0x1::Account;
    use 0x1::PackageTxnManager;
    use 0x1::BlockReward;
    use 0x1::Block;
    use 0x1::STC::STC;
    use 0x1::TransactionFee;
    use 0x1::Timestamp;
    use 0x1::ChainId;
    use 0x1::Errors;
    use 0x1::TransactionPublishOption;
    use 0x1::Epoch;
    use 0x1::Hash;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    const TXN_PAYLOAD_TYPE_SCRIPT: u8 = 0;
    const TXN_PAYLOAD_TYPE_PACKAGE: u8 = 1;

    const EPROLOGUE_ACCOUNT_DOES_NOT_EXIST: u64 = 0;
    const EPROLOGUE_TRANSACTION_EXPIRED: u64 = 5;
    const EPROLOGUE_BAD_CHAIN_ID: u64 = 6;
    const EPROLOGUE_MODULE_NOT_ALLOWED: u64 = 7;
    const EPROLOGUE_SCRIPT_NOT_ALLOWED: u64 = 8;


    // The prologue is invoked at the beginning of every transaction
    // It verifies:
    // - The account's auth key matches the transaction's public key
    // - That the account has enough balance to pay for all of the gas
    // - That the sequence number matches the transaction's sequence key
    public fun prologue<TokenType>(
        account: &signer,
        txn_sender: address,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        txn_payload_type: u8,
        txn_script_or_package_hash: vector<u8>,
        txn_package_address: address,
    ) {
        // Can only be invoked by genesis account
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            Errors::requires_address(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST),
        );
        // Check that the chain ID stored on-chain matches the chain ID
        // specified by the transaction
        assert(ChainId::get() == chain_id, Errors::invalid_argument(EPROLOGUE_BAD_CHAIN_ID));
        Account::txn_prologue<TokenType>(
            account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units,
        );
        assert(
            TransactionTimeout::is_valid_transaction_timestamp(txn_expiration_time),
            Errors::invalid_argument(EPROLOGUE_TRANSACTION_EXPIRED),
        );
        if (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE) {
            assert(
                TransactionPublishOption::is_module_allowed(Signer::address_of(account)),
                Errors::invalid_argument(EPROLOGUE_MODULE_NOT_ALLOWED),
            );
            PackageTxnManager::package_txn_prologue(
                account,
                txn_sender,
                txn_package_address,
                txn_script_or_package_hash,
            );
        } else if (txn_payload_type == TXN_PAYLOAD_TYPE_SCRIPT) {
            assert(
                TransactionPublishOption::is_script_allowed(
                    Signer::address_of(account),
                    &txn_script_or_package_hash,
                ),
                Errors::invalid_argument(EPROLOGUE_SCRIPT_NOT_ALLOWED),
            );
        };
    }

    spec fun prologue {
        aborts_if Signer::address_of(account) != CoreAddresses::GENESIS_ADDRESS();
        aborts_if !exists<ChainId::ChainId>(CoreAddresses::GENESIS_ADDRESS());
        aborts_if ChainId::get() != chain_id;
        aborts_if !exists<Account::Account>(txn_sender);
        aborts_if Hash::sha3_256(txn_public_key) != global<Account::Account>(txn_sender).authentication_key;
        aborts_if txn_gas_price * txn_max_gas_units > max_u64();
        include Timestamp::AbortsIfTimestampNotExists;
        include Block::AbortsIfBlockMetadataNotExist;
        aborts_if !exists<Account::Balance<TokenType>>(txn_sender);
        aborts_if global<Account::Balance<TokenType>>(txn_sender).token.value < txn_gas_price * txn_max_gas_units;
        aborts_if txn_sequence_number < global<Account::Account>(txn_sender).sequence_number;
        aborts_if txn_sequence_number != global<Account::Account>(txn_sender).sequence_number;
        include TransactionTimeout::AbortsIfTimestampNotValid;
    }

    // The epilogue is invoked at the end of transactions.
    // It collects gas and bumps the sequence number
    public fun epilogue<TokenType>(
        account: &signer,
        txn_sender: address,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
        txn_payload_type: u8,
        _txn_script_or_package_hash: vector<u8>,
        txn_package_address: address,
        // txn execute success or fail.
        success: bool,
    ) {
        CoreAddresses::assert_genesis_address(account);
        Account::txn_epilogue<TokenType>(
            account,
            txn_sender,
            txn_sequence_number,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining,
        );
        if (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE) {
            PackageTxnManager::package_txn_epilogue(
                account,
                txn_sender,
                txn_package_address,
                success,
            );
        }
    }

    spec fun epilogue {
        include CoreAddresses::AbortsIfNotGenesisAddress;
        pragma verify = false;
    }

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    public fun block_prologue(
        account: &signer,
        parent_hash: vector<u8>,
        timestamp: u64,
        author: address,
        auth_key_vec: vector<u8>,
        uncles: u64,
        number: u64,
        chain_id: u8,
        parent_gas_used: u64,
    ) {
        // Can only be invoked by genesis account
        CoreAddresses::assert_genesis_address(account);
        Timestamp::update_global_time(account, timestamp);
        // Check that the chain ID stored on-chain matches the chain ID
        // specified by the transaction
        assert(ChainId::get() == chain_id, Errors::invalid_argument(EPROLOGUE_BAD_CHAIN_ID));
        //get previous author for distribute txn_fee
        let previous_author = Block::get_current_author();
        let txn_fee = TransactionFee::distribute_transaction_fees<STC>(account);
        distribute(txn_fee, previous_author);
        Block::process_block_metadata(
            account,
            parent_hash,
            author,
            timestamp,
            uncles,
            number,
        );
        let reward = Epoch::adjust_epoch(account, number, timestamp, uncles, parent_gas_used);
        BlockReward::process_block_reward(account, number, reward, author, auth_key_vec);
    }

    spec fun block_prologue {
        pragma verify = false;
    }

    fun distribute<TokenType>(txn_fee: Token<TokenType>, author: address) {
        let value = Token::value<TokenType>(&txn_fee);
        if (value > 0) {
            Account::deposit<TokenType>(author, txn_fee);
        } else {
            Token::destroy_zero<TokenType>(txn_fee);
        }
    }

    spec fun distribute {
        pragma verify = false;
    }
}
}