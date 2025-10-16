/// Per-address gas fee freeze bucket. Each address may own one per TokenType.
/// Transactions deposit gas fees here during epilogue; at block end, fees are
/// swept into the global TransactionFee bucket for distribution.
module starcoin_framework::stc_gas_fee_freeze {
    friend starcoin_framework::stc_transaction_fee;
    use std::vector;
    use starcoin_std::table::{Self, Table};
    use starcoin_framework::coin;
    use starcoin_framework::create_signer;
    use starcoin_framework::system_addresses;

    /// Per-address freeze bucket for a given TokenType, lives under the payer's address.
    struct TxnGasFreeze<phantom TokenType> has key { amount: coin::Coin<TokenType> }

    /// Per-block index of addresses that deposited fees during the block, stored under @starcoin_framework.
    /// We keep both a vector for iteration order and a table for O(1) membership checks to avoid duplicates.
    struct FreezeIndex<phantom TokenType> has key {
        addrs: vector<address>,
        present: Table<address, bool>,
    }

    /// Publish FreezeIndex for TokenType under @starcoin_framework if absent.
    public(friend) fun init_index<TokenType>(account: &signer) acquires FreezeIndex {
        system_addresses::assert_starcoin_framework(account);
        if (!exists<FreezeIndex<TokenType>>(system_addresses::get_starcoin_framework())) {
            move_to<FreezeIndex<TokenType>>(account, FreezeIndex<TokenType> { addrs: vector::empty<address>(), present: table::new<address, bool>() });
        }
    }

    /// Ensure a TxnGasFreeze resource exists under `addr` for TokenType.
    fun ensure_bucket<TokenType>(addr: address) acquires TxnGasFreeze {
        if (!exists<TxnGasFreeze<TokenType>>(addr)) {
            let s = create_signer::create_signer(addr);
            move_to<TxnGasFreeze<TokenType>>(&s, TxnGasFreeze<TokenType> { amount: coin::zero<TokenType>() });
        }
    }

    /// Record `addr` into the FreezeIndex if not yet present.
    fun index_addr_if_needed<TokenType>(addr: address) acquires FreezeIndex {
        let idx = &mut borrow_global_mut<FreezeIndex<TokenType>>(system_addresses::get_starcoin_framework());
        if (!table::contains(&idx.present, addr)) {
            table::add(&mut idx.present, addr, true);
            vector::push_back(&mut idx.addrs, addr);
        }
    }

    /// Deposit `fee` into the freeze bucket of `addr` and index the address in the current block.
    public(friend) fun deposit<TokenType>(addr: address, fee: coin::Coin<TokenType>) acquires TxnGasFreeze, FreezeIndex {
        ensure_bucket<TokenType>(addr);
        index_addr_if_needed<TokenType>(addr);
        let bucket = &mut borrow_global_mut<TxnGasFreeze<TokenType>>(addr).amount;
        coin::merge(bucket, fee);
    }

    /// Drain all frozen fees for all indexed addresses and clear the index. Framework only.
    public(friend) fun drain_index_all<TokenType>(account: &signer): coin::Coin<TokenType> acquires TxnGasFreeze, FreezeIndex {
        system_addresses::assert_starcoin_framework(account);
        let idx_addr = system_addresses::get_starcoin_framework();
        let idx = &mut borrow_global_mut<FreezeIndex<TokenType>>(idx_addr);

        let acc = coin::zero<TokenType>();
        // Iterate in reverse by popping for O(1)
        let addrs = &mut idx.addrs;
        while (!vector::is_empty(addrs)) {
            let addr = vector::pop_back(addrs);
            // remove from present map if exists
            if (table::contains(&idx.present, addr)) { let _ = table::remove(&mut idx.present, addr); };
            if (exists<TxnGasFreeze<TokenType>>(addr)) {
                let bucket = &mut borrow_global_mut<TxnGasFreeze<TokenType>>(addr).amount;
                let drained = coin::extract_all(bucket);
                coin::merge(&mut acc, drained);
            };
        };
        acc
    }
}
