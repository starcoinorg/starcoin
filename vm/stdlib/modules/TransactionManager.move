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
    use 0x1::ErrorCode;
    use 0x1::TransactionPublishOption;

    const TXN_PAYLOAD_TYPE_SCRIPT: u8 = 0;
    const TXN_PAYLOAD_TYPE_PACKAGE: u8 = 1;

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
            ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST(),
        );
        // Check that the chain ID stored on-chain matches the chain ID
        // specified by the transaction
        assert(ChainId::get() == chain_id, ErrorCode::PROLOGUE_BAD_CHAIN_ID());
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
            ErrorCode::PROLOGUE_TRANSACTION_EXPIRED(),
        );
        if (txn_payload_type == TXN_PAYLOAD_TYPE_PACKAGE) {
            assert(
                TransactionPublishOption::is_module_allowed(Signer::address_of(account)),
                ErrorCode::PROLOGUE_MODULE_NOT_ALLOWED(),
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
                ErrorCode::PROLOGUE_SCRIPT_NOT_ALLOWED(),
            );
        };
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
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            ErrorCode::ENOT_GENESIS_ACCOUNT(),
        );
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

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    public fun block_prologue(
        account: &signer,
        parent_hash: vector<u8>,
        timestamp: u64,
        author: address,
        public_key_vec: vector<u8>,
        uncles: u64,
        number: u64,
        chain_id: u8,
    ) {
        // Can only be invoked by genesis account
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            ErrorCode::ENOT_GENESIS_ACCOUNT(),
        );
        Timestamp::update_global_time(account, timestamp);
        // Check that the chain ID stored on-chain matches the chain ID
        // specified by the transaction
        assert(ChainId::get() == chain_id, ErrorCode::PROLOGUE_BAD_CHAIN_ID());
        //get previous author for distribute txn_fee
        let previous_author = Block::get_current_author();
        let txn_fee = TransactionFee::distribute_transaction_fees<STC>(account);
        distribute(account, txn_fee, previous_author);
        let reward = Block::process_block_metadata(
            account,
            parent_hash,
            author,
            timestamp,
            uncles,
            number,
        );
        BlockReward::process_block_reward(account, number, reward, author, public_key_vec);
    }

    fun distribute<TokenType>(account: &signer, txn_fee: Token<TokenType>, author: address) {
        let value = Token::value<TokenType>(&txn_fee);
        if (value > 0) {
            Account::deposit_to<TokenType>(account, author, txn_fee);
        } else {
            Token::destroy_zero<TokenType>(txn_fee);
        }
    }
}
}