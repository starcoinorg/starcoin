/// The module for the Treasury of DAO, which can hold the token of DAO.
module starcoin_framework::treasury_fa {
    use std::error;
    use std::option;
    use std::signer;
    use starcoin_framework::create_signer::create_signer;

    use starcoin_framework::fungible_asset;
    use starcoin_framework::object;
    use starcoin_framework::object::{Object};
    use starcoin_framework::fungible_asset::{FungibleStore, FungibleAsset, TransferRef};

    use starcoin_framework::account;
    use starcoin_framework::coin;
    use starcoin_framework::event;
    use starcoin_framework::timestamp;

    use starcoin_std::math128;

    struct Treasury<phantom CoinT> has store, key {
        fa_store: Object<FungibleStore>,
        store_owner: address,
        /// event handle for treasury withdraw event
        withdraw_events: event::EventHandle<WithdrawEvent>,
        /// event handle for treasury deposit event
        deposit_events: event::EventHandle<DepositEvent>,
    }

    /// A withdraw capability allows tokens of type `CoinT` to be withdraw from Treasury.
    struct WithdrawCapability<phantom CoinT> has key, store {
        owner: address,
    }

    /// A linear time withdraw capability which can withdraw token from Treasury in a period by time-based linear release.
    struct LinearWithdrawCapability<phantom CoinT> has key, store {
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


    /// Init a Treasury for CoinT. Can only be called by token issuer.
    public fun initialize<CoinT>(
        account: &signer,
        initia_fa: FungibleAsset,
        _transfer_ref: &TransferRef
    ): WithdrawCapability<CoinT> {
        let coin_metadata_opt = coin::paired_metadata<CoinT>();
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
        move_to<Treasury<CoinT>>(account, Treasury {
            fa_store,
            store_owner: object::address_from_constructor_ref(&constructor_ref),
            withdraw_events: account::new_event_handle<WithdrawEvent>(account),
            deposit_events: account::new_event_handle<DepositEvent>(account),
        });

        WithdrawCapability<CoinT> {
            owner: signer::address_of(account),
        }
    }

    /// Check the Treasury of CoinT is exists.
    public fun exists_at<CoinT>(owner: address): bool acquires Treasury {
        exists<Treasury<CoinT>>(owner);

        let treasury = borrow_global<Treasury<CoinT>>(owner);
        fungible_asset::store_exists(object::owner(treasury.fa_store))
    }

    /// Get the balance of CoinT's Treasury
    /// if the Treasury do not exists, return 0.
    public fun balance<CoinT>(owner: address): u128 acquires Treasury {
        if (!exists<Treasury<CoinT>>(owner)) {
            return 0
        };
        let treasury = borrow_global<Treasury<CoinT>>(owner);
        (fungible_asset::balance(treasury.fa_store) as u128)
    }

    public fun deposit<CoinT>(owner: address, fa: FungibleAsset) acquires Treasury {
        assert!(exists_at<Treasury<CoinT>>(owner), error::not_found(ERR_TREASURY_NOT_EXIST));

        let treasury = borrow_global_mut<Treasury<CoinT>>(owner);

        let amount = fungible_asset::amount(&fa);
        fungible_asset::deposit(treasury.fa_store, fa);
        event::emit_event(
            &mut treasury.deposit_events,
            DepositEvent {
                amount: (amount as u128)
            },
        );
    }

    fun inner_do_withdraw<CoinT>(owner: address, amount: u128): FungibleAsset acquires Treasury {
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        assert!(exists_at<Treasury<CoinT>>(owner), error::not_found(ERR_TREASURY_NOT_EXIST));

        let treasury = borrow_global_mut<Treasury<CoinT>>(owner);
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
    public fun withdraw_with_capability<CoinT>(
        cap: &mut WithdrawCapability<CoinT>,
        amount: u128,
    ): FungibleAsset acquires Treasury {
        inner_do_withdraw<CoinT>(cap.owner, amount)
    }

    /// Withdraw from CoinT's treasury, the signer must have WithdrawCapability<CoinT>
    public fun withdraw<CoinT>(
        signer: &signer,
        amount: u128
    ): FungibleAsset acquires Treasury, WithdrawCapability {
        let cap = borrow_global_mut<WithdrawCapability<CoinT>>(signer::address_of(signer));
        Self::withdraw_with_capability(cap, amount)
    }

    /// Issue a `LinearWithdrawCapability` with given `WithdrawCapability`.
    public fun issue_linear_withdraw_capability<CoinT>(
        cap: &mut WithdrawCapability<CoinT>,
        amount: u128,
        period: u64
    ): LinearWithdrawCapability<CoinT> {
        assert!(period > 0, error::invalid_argument(ERR_INVALID_PERIOD));
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        let start_time = timestamp::now_seconds();
        LinearWithdrawCapability<CoinT> {
            owner: cap.owner,
            total: amount,
            withdraw: 0,
            start_time,
            period,
        }
    }


    /// Withdraw tokens with given `LinearWithdrawCapability`.
    public fun withdraw_with_linear_capability<CoinT>(
        cap: &mut LinearWithdrawCapability<CoinT>,
    ): FungibleAsset acquires Treasury {
        let amount = withdraw_amount_of_linear_cap(cap);
        let fa = Self::inner_do_withdraw<CoinT>(cap.owner, amount);
        cap.withdraw = cap.withdraw + amount;
        fa
    }

    /// Split the given `LinearWithdrawCapability`.
    public fun split_linear_withdraw_cap<CoinT>(
        cap: &mut LinearWithdrawCapability<CoinT>,
        amount: u128,
    ): (FungibleAsset, LinearWithdrawCapability<CoinT>) acquires Treasury {
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        let token = Self::withdraw_with_linear_capability(cap);
        assert!((cap.withdraw + amount) <= cap.total, error::invalid_argument(ERR_TOO_BIG_AMOUNT));
        cap.total = cap.total - amount;
        let start_time = timestamp::now_seconds();
        let new_period = cap.start_time + cap.period - start_time;
        let new_key = LinearWithdrawCapability<CoinT> {
            owner: cap.owner,
            total: amount,
            withdraw: 0,
            start_time,
            period: new_period
        };
        (token, new_key)
    }

    /// Returns the amount of the LinearWithdrawCapability can mint now.
    public fun withdraw_amount_of_linear_cap<CoinT>(cap: &LinearWithdrawCapability<CoinT>): u128 {
        let now = timestamp::now_seconds();
        let elapsed_time = now - cap.start_time;
        if (elapsed_time >= cap.period) {
            cap.total - cap.withdraw
        } else {
            math128::mul_div(cap.total, (elapsed_time as u128), (cap.period as u128)) - cap.withdraw
        }
    }


    /// Check if the given `LinearWithdrawCapability` is empty.
    public fun is_empty_linear_withdraw_cap<CoinT>(key: &LinearWithdrawCapability<CoinT>): bool {
        key.total == key.withdraw
    }

    /// Remove mint capability from `signer`.
    public fun remove_withdraw_capability<CoinT>(
        signer: &signer
    ): WithdrawCapability<CoinT> acquires WithdrawCapability {
        move_from<WithdrawCapability<CoinT>>(signer::address_of(signer))
    }


    /// Save mint capability to `signer`.
    public fun add_withdraw_capability<CoinT>(signer: &signer, cap: WithdrawCapability<CoinT>) {
        move_to(signer, cap)
    }


    /// Destroy the given mint capability.
    public fun destroy_withdraw_capability<CoinT>(cap: WithdrawCapability<CoinT>) {
        let WithdrawCapability<CoinT> { owner: _ } = cap;
    }


    /// Add LinearWithdrawCapability to `signer`, a address only can have one LinearWithdrawCapability<T>
    public fun add_linear_withdraw_capability<CoinT>(signer: &signer, cap: LinearWithdrawCapability<CoinT>) {
        move_to(signer, cap)
    }


    /// Remove LinearWithdrawCapability from `signer`.
    public fun remove_linear_withdraw_capability<CoinT>(
        signer: &signer
    ): LinearWithdrawCapability<CoinT> acquires LinearWithdrawCapability {
        move_from<LinearWithdrawCapability<CoinT>>(signer::address_of(signer))
    }

    /// Destroy LinearWithdrawCapability.
    public fun destroy_linear_withdraw_capability<CoinT>(cap: LinearWithdrawCapability<CoinT>) {
        let LinearWithdrawCapability { owner: _, total: _, withdraw: _, start_time: _, period: _ } = cap;
    }

    public fun is_empty_linear_withdraw_capability<CoinT>(cap: &LinearWithdrawCapability<CoinT>): bool {
        cap.total == cap.withdraw
    }

    /// Get LinearWithdrawCapability total amount
    public fun get_linear_withdraw_capability_total<CoinT>(cap: &LinearWithdrawCapability<CoinT>): u128 {
        cap.total
    }

    /// Get LinearWithdrawCapability withdraw amount
    public fun get_linear_withdraw_capability_withdraw<CoinT>(cap: &LinearWithdrawCapability<CoinT>): u128 {
        cap.withdraw
    }

    /// Get LinearWithdrawCapability period in seconds
    public fun get_linear_withdraw_capability_period<CoinT>(cap: &LinearWithdrawCapability<CoinT>): u64 {
        cap.period
    }

    /// Get LinearWithdrawCapability start_time in seconds
    public fun get_linear_withdraw_capability_start_time<CoinT>(cap: &LinearWithdrawCapability<CoinT>): u64 {
        cap.start_time
    }
}