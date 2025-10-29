/// The module for the Treasury of DAO, which can hold the token of DAO.
module starcoin_framework::treasury {
    use std::error;
    use std::option;
    use std::signer;

    use starcoin_framework::account;
    use starcoin_framework::coin;
    use starcoin_framework::create_signer::create_signer;
    use starcoin_framework::event;
    use starcoin_framework::fungible_asset;
    use starcoin_framework::fungible_asset::{FungibleAsset, FungibleStore};
    use starcoin_framework::object;
    use starcoin_framework::object::Object;
    use starcoin_framework::timestamp;

    use starcoin_std::debug;
    use starcoin_std::math128;

    struct Treasury<phantom TokenT> has store, key {
        fa_store: Object<FungibleStore>,
        store_owner: address,
        /// event handle for treasury withdraw event
        withdraw_events: event::EventHandle<WithdrawEvent>,
        /// event handle for treasury deposit event
        deposit_events: event::EventHandle<DepositEvent>,
    }

    /// A withdraw capability allows tokens of type `TokenT` to be withdraw from Treasury.
    struct WithdrawCapability<phantom TokenT> has key, store {
        owner: address,
    }

    /// A linear time withdraw capability which can withdraw token from Treasury in a period by time-based linear release.
    struct LinearWithdrawCapability<phantom TokenT> has key, store {
        owner: address,

        /// The total amount of tokens that can be withdrawn by this capability
        total: u128,
        /// The amount of tokens that have been withdrawn by this capability
        withdraw: u128,
        /// The time-based linear release start time, timestamp in seconds.
        start_time: u64,
        ///  The time-based linear release period in seconds
        period: u64,
    }

    /// Message for treasury withdraw event.
    struct WithdrawEvent has drop, store {
        amount: u128,
    }

    /// Message for treasury deposit event.
    struct DepositEvent has drop, store {
        amount: u128,
    }

    const ERR_INVALID_PERIOD: u64 = 101;
    const ERR_ZERO_AMOUNT: u64 = 102;
    const ERR_TOO_BIG_AMOUNT: u64 = 103;
    const ERR_NOT_AUTHORIZED: u64 = 104;
    const ERR_TREASURY_NOT_EXIST: u64 = 105;
    const ERR_TOKEN_NOT_CREATE_TOKEN_PAIR: u64 = 106;
    const ERR_INITA_ASSET_NOT_MATCH: u64 = 107;
    const ERR_TREASURY_INIAIZLIED: u64 = 108;

    /// Init a Treasury for TokenT. Can only be called by token issuer.
    public fun initialize<TokenT>(
        account: &signer,
        initia_fa: FungibleAsset,
    ): WithdrawCapability<TokenT> {
        debug::print(&std::string::utf8(b"treasury::initialize | Entered"));
        debug::print(account);

        let account_addr = signer::address_of(account);
        assert!(!exists_at<TokenT>(account_addr), error::invalid_state(ERR_TREASURY_INIAIZLIED));

        let coin_metadata_opt = coin::paired_metadata<TokenT>();
        assert!(option::is_some(&coin_metadata_opt), error::invalid_state(ERR_TOKEN_NOT_CREATE_TOKEN_PAIR));

        let asset_metadata = fungible_asset::asset_metadata(&initia_fa);
        assert!(
            asset_metadata == option::destroy_some(coin_metadata_opt),
            error::invalid_state(ERR_INITA_ASSET_NOT_MATCH)
        );

        let constructor_ref = object::create_object(signer::address_of(account));
        let fa_store = fungible_asset::create_store(&constructor_ref, asset_metadata);

        fungible_asset::deposit(fa_store, initia_fa);

        // Check fungible asset
        move_to(account, Treasury<TokenT> {
            fa_store,
            store_owner: object::address_from_constructor_ref(&constructor_ref),
            withdraw_events: account::new_event_handle<WithdrawEvent>(account),
            deposit_events: account::new_event_handle<DepositEvent>(account),
        });

        let cap = WithdrawCapability<TokenT> {
            owner: signer::address_of(account),
        };

        debug::print(&std::string::utf8(b"treasury::initialize | Exited"));
        cap
    }

    /// Check the Treasury of TokenT is exists.
    public fun exists_at<TokenT>(owner: address): bool {
        exists<Treasury<TokenT>>(owner)
    }

    /// Get the balance of TokenT's Treasury
    /// if the Treasury do not exists, return 0.
    public fun balance<TokenT>(owner: address): u128 acquires Treasury {
        if (!exists<Treasury<TokenT>>(owner)) {
            return 0
        };
        let treasury = borrow_global<Treasury<TokenT>>(owner);
        (fungible_asset::balance(treasury.fa_store) as u128)
    }

    public fun deposit<TokenT>(owner: address, fa: FungibleAsset) acquires Treasury {
        assert!(exists_at<TokenT>(owner), error::not_found(ERR_TREASURY_NOT_EXIST));

        let treasury = borrow_global_mut<Treasury<TokenT>>(owner);

        let amount = fungible_asset::amount(&fa);
        fungible_asset::deposit(treasury.fa_store, fa);
        event::emit_event(
            &mut treasury.deposit_events,
            DepositEvent {
                amount: (amount as u128)
            },
        );
    }

    fun inner_do_withdraw<TokenT>(owner: address, amount: u128): FungibleAsset acquires Treasury {
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        assert!(exists_at<TokenT>(owner), error::not_found(ERR_TREASURY_NOT_EXIST));

        let treasury = borrow_global_mut<Treasury<TokenT>>(owner);
        assert!(
            amount <= (fungible_asset::balance(treasury.fa_store) as u128),
            error::invalid_argument(ERR_TOO_BIG_AMOUNT)
        );
        event::emit_event(
            &mut treasury.withdraw_events,
            WithdrawEvent { amount },
        );
        let store_signer = create_signer(object::owner(treasury.fa_store));
        fungible_asset::withdraw(&store_signer, treasury.fa_store, (amount as u64))
    }

    /// Withdraw tokens with given `LinearWithdrawCapability`.
    public fun withdraw_with_capability<TokenT>(
        cap: &mut WithdrawCapability<TokenT>,
        amount: u128,
    ): FungibleAsset acquires Treasury {
        inner_do_withdraw<TokenT>(cap.owner, amount)
    }

    /// Withdraw from TokenT's treasury, the signer must have WithdrawCapability<TokenT>
    public fun withdraw<TokenT>(
        signer: &signer,
        amount: u128
    ): FungibleAsset acquires Treasury, WithdrawCapability {
        let cap = borrow_global_mut<WithdrawCapability<TokenT>>(signer::address_of(signer));
        Self::withdraw_with_capability(cap, amount)
    }

    /// Issue a `LinearWithdrawCapability` with given `WithdrawCapability`.
    public fun issue_linear_withdraw_capability<TokenT>(
        cap: &mut WithdrawCapability<TokenT>,
        amount: u128,
        period: u64
    ): LinearWithdrawCapability<TokenT> {
        assert!(period > 0, error::invalid_argument(ERR_INVALID_PERIOD));
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        let start_time = timestamp::now_seconds();
        LinearWithdrawCapability<TokenT> {
            owner: cap.owner,
            total: amount,
            withdraw: 0,
            start_time,
            period,
        }
    }


    /// Withdraw tokens with given `LinearWithdrawCapability`.
    public fun withdraw_with_linear_capability<TokenT>(
        cap: &mut LinearWithdrawCapability<TokenT>,
    ): FungibleAsset acquires Treasury {
        let amount = withdraw_amount_of_linear_cap(cap);
        let fa = Self::inner_do_withdraw<TokenT>(cap.owner, amount);
        cap.withdraw = cap.withdraw + amount;
        fa
    }

    /// Split the given `LinearWithdrawCapability`.
    public fun split_linear_withdraw_cap<TokenT>(
        cap: &mut LinearWithdrawCapability<TokenT>,
        amount: u128,
    ): (FungibleAsset, LinearWithdrawCapability<TokenT>) acquires Treasury {
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        let token = Self::withdraw_with_linear_capability(cap);
        assert!((cap.withdraw + amount) <= cap.total, error::invalid_argument(ERR_TOO_BIG_AMOUNT));
        cap.total = cap.total - amount;
        let start_time = timestamp::now_seconds();
        let new_period = cap.start_time + cap.period - start_time;
        let new_key = LinearWithdrawCapability<TokenT> {
            owner: cap.owner,
            total: amount,
            withdraw: 0,
            start_time,
            period: new_period
        };
        (token, new_key)
    }

    /// Returns the amount of the LinearWithdrawCapability can mint now.
    public fun withdraw_amount_of_linear_cap<TokenT>(cap: &LinearWithdrawCapability<TokenT>): u128 {
        let now = timestamp::now_seconds();
        let elapsed_time = now - cap.start_time;
        if (elapsed_time >= cap.period) {
            cap.total - cap.withdraw
        } else {
            math128::mul_div(cap.total, (elapsed_time as u128), (cap.period as u128)) - cap.withdraw
        }
    }


    /// Check if the given `LinearWithdrawCapability` is empty.
    public fun is_empty_linear_withdraw_cap<TokenT>(key: &LinearWithdrawCapability<TokenT>): bool {
        key.total == key.withdraw
    }

    /// Remove mint capability from `signer`.
    public fun remove_withdraw_capability<TokenT>(
        signer: &signer
    ): WithdrawCapability<TokenT> acquires WithdrawCapability {
        move_from<WithdrawCapability<TokenT>>(signer::address_of(signer))
    }

    /// Save mint capability to `signer`.
    public fun add_withdraw_capability<TokenT>(signer: &signer, cap: WithdrawCapability<TokenT>) {
        move_to(signer, cap)
    }


    /// Destroy the given mint capability.
    public fun destroy_withdraw_capability<TokenT>(cap: WithdrawCapability<TokenT>) {
        let WithdrawCapability<TokenT> { owner: _ } = cap;
    }


    /// Add LinearWithdrawCapability to `signer`, a address only can have one LinearWithdrawCapability<T>
    public fun add_linear_withdraw_capability<TokenT>(signer: &signer, cap: LinearWithdrawCapability<TokenT>) {
        move_to(signer, cap)
    }


    /// Remove LinearWithdrawCapability from `signer`.
    public fun remove_linear_withdraw_capability<TokenT>(
        signer: &signer
    ): LinearWithdrawCapability<TokenT> acquires LinearWithdrawCapability {
        move_from<LinearWithdrawCapability<TokenT>>(signer::address_of(signer))
    }

    /// Destroy LinearWithdrawCapability.
    public fun destroy_linear_withdraw_capability<TokenT>(cap: LinearWithdrawCapability<TokenT>) {
        let LinearWithdrawCapability { owner: _, total: _, withdraw: _, start_time: _, period: _ } = cap;
    }

    public fun is_empty_linear_withdraw_capability<TokenT>(cap: &LinearWithdrawCapability<TokenT>): bool {
        cap.total == cap.withdraw
    }

    /// Get LinearWithdrawCapability total amount
    public fun get_linear_withdraw_capability_total<TokenT>(cap: &LinearWithdrawCapability<TokenT>): u128 {
        cap.total
    }

    /// Get LinearWithdrawCapability withdraw amount
    public fun get_linear_withdraw_capability_withdraw<TokenT>(cap: &LinearWithdrawCapability<TokenT>): u128 {
        cap.withdraw
    }

    /// Get LinearWithdrawCapability period in seconds
    public fun get_linear_withdraw_capability_period<TokenT>(cap: &LinearWithdrawCapability<TokenT>): u64 {
        cap.period
    }

    /// Get LinearWithdrawCapability start_time in seconds
    public fun get_linear_withdraw_capability_start_time<TokenT>(cap: &LinearWithdrawCapability<TokenT>): u64 {
        cap.start_time
    }
}