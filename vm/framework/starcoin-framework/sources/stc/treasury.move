/// The module for the Treasury of DAO, which can hold the token of DAO.
module starcoin_framework::treasury {
    use std::error;
    use std::signer;
    use starcoin_std::math128;
    use starcoin_std::type_info;
    use starcoin_framework::timestamp;
    use starcoin_framework::stc_util;
    use starcoin_framework::account;
    use starcoin_framework::event;
    use starcoin_framework::coin;


    struct Treasury<phantom TokenT> has store, key {
        balance: coin::Coin<TokenT>,
        /// event handle for treasury withdraw event
        withdraw_events: event::EventHandle<WithdrawEvent>,
        /// event handle for treasury deposit event
        deposit_events: event::EventHandle<DepositEvent>,
    }

    /// A withdraw capability allows tokens of type `TokenT` to be withdraw from Treasury.
    struct WithdrawCapability<phantom TokenT> has key, store {}

    /// A linear time withdraw capability which can withdraw token from Treasury in a period by time-based linear release.
    struct LinearWithdrawCapability<phantom TokenT> has key, store {
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


    /// Init a Treasury for TokenT. Can only be called by token issuer.
    public fun initialize<TokenT: store>(signer: &signer, init_token: coin::Coin<TokenT>): WithdrawCapability<TokenT> {
        let token_issuer = stc_util::token_issuer<TokenT>();
        assert!(signer::address_of(signer) == token_issuer, error::invalid_state(ERR_NOT_AUTHORIZED));
        let treasure = Treasury {
            balance: init_token,
            withdraw_events: account::new_event_handle<WithdrawEvent>(signer),
            deposit_events: account::new_event_handle<DepositEvent>(signer),
        };
        move_to(signer, treasure);
        WithdrawCapability<TokenT> {}
    }


    /// Check the Treasury of TokenT is exists.
    public fun exists_at<TokenT: store>(): bool {
        let token_issuer = type_info::account_address(&type_info::type_of<TokenT>());
        exists<Treasury<TokenT>>(token_issuer)
    }

    /// Get the balance of TokenT's Treasury
    /// if the Treasury do not exists, return 0.
    public fun balance<TokenT: store>(): u128 acquires Treasury {
        let token_issuer = stc_util::token_issuer<TokenT>();
        if (!exists<Treasury<TokenT>>(token_issuer)) {
            return 0
        };
        let treasury = borrow_global<Treasury<TokenT>>(token_issuer);
        (coin::value(&treasury.balance) as u128)
    }

    public fun deposit<TokenT: store>(token: coin::Coin<TokenT>) acquires Treasury {
        assert!(exists_at<TokenT>(), error::not_found(ERR_TREASURY_NOT_EXIST));
        let token_address = stc_util::token_issuer<TokenT>();
        let treasury = borrow_global_mut<Treasury<TokenT>>(token_address);
        let amount = coin::value(&token);
        event::emit_event(
            &mut treasury.deposit_events,
            DepositEvent {
                amount: (amount as u128)
            },
        );
        coin::merge(&mut treasury.balance, token);
    }

    fun do_withdraw<TokenT: store>(amount: u128): coin::Coin<TokenT> acquires Treasury {
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        assert!(exists_at<TokenT>(), error::not_found(ERR_TREASURY_NOT_EXIST));
        let token_address = stc_util::token_issuer<TokenT>();
        let treasury = borrow_global_mut<Treasury<TokenT>>(token_address);
        assert!(amount <= (coin::value(&treasury.balance) as u128), error::invalid_argument(ERR_TOO_BIG_AMOUNT));
        event::emit_event(
            &mut treasury.withdraw_events,
            WithdrawEvent { amount },
        );
        coin::extract(&mut treasury.balance, (amount as u64))
    }

    /// Withdraw tokens with given `LinearWithdrawCapability`.
    public fun withdraw_with_capability<TokenT: store>(
        _cap: &mut WithdrawCapability<TokenT>,
        amount: u128,
    ): coin::Coin<TokenT> acquires Treasury {
        do_withdraw(amount)
    }


    /// Withdraw from TokenT's treasury, the signer must have WithdrawCapability<TokenT>
    public fun withdraw<TokenT: store>(
        signer: &signer,
        amount: u128
    ): coin::Coin<TokenT> acquires Treasury, WithdrawCapability {
        let cap = borrow_global_mut<WithdrawCapability<TokenT>>(signer::address_of(signer));
        Self::withdraw_with_capability(cap, amount)
    }

    /// Issue a `LinearWithdrawCapability` with given `WithdrawCapability`.
    public fun issue_linear_withdraw_capability<TokenT: store>(
        _capability: &mut WithdrawCapability<TokenT>,
        amount: u128,
        period: u64
    ): LinearWithdrawCapability<TokenT> {
        assert!(period > 0, error::invalid_argument(ERR_INVALID_PERIOD));
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        let start_time = timestamp::now_seconds();
        LinearWithdrawCapability<TokenT> {
            total: amount,
            withdraw: 0,
            start_time,
            period,
        }
    }


    /// Withdraw tokens with given `LinearWithdrawCapability`.
    public fun withdraw_with_linear_capability<TokenT: store>(
        cap: &mut LinearWithdrawCapability<TokenT>,
    ): coin::Coin<TokenT> acquires Treasury {
        let amount = withdraw_amount_of_linear_cap(cap);
        let token = do_withdraw(amount);
        cap.withdraw = cap.withdraw + amount;
        token
    }

    /// Withdraw from TokenT's  treasury, the signer must have LinearWithdrawCapability<TokenT>
    public fun withdraw_by_linear<TokenT: store>(
        signer: &signer,
    ): coin::Coin<TokenT> acquires Treasury, LinearWithdrawCapability {
        let cap = borrow_global_mut<LinearWithdrawCapability<TokenT>>(signer::address_of(signer));
        Self::withdraw_with_linear_capability(cap)
    }


    /// Split the given `LinearWithdrawCapability`.
    public fun split_linear_withdraw_cap<TokenT: store>(
        cap: &mut LinearWithdrawCapability<TokenT>,
        amount: u128,
    ): (coin::Coin<TokenT>, LinearWithdrawCapability<TokenT>) acquires Treasury {
        assert!(amount > 0, error::invalid_argument(ERR_ZERO_AMOUNT));
        let token = Self::withdraw_with_linear_capability(cap);
        assert!((cap.withdraw + amount) <= cap.total, error::invalid_argument(ERR_TOO_BIG_AMOUNT));
        cap.total = cap.total - amount;
        let start_time = timestamp::now_seconds();
        let new_period = cap.start_time + cap.period - start_time;
        let new_key = LinearWithdrawCapability<TokenT> {
            total: amount,
            withdraw: 0,
            start_time,
            period: new_period
        };
        (token, new_key)
    }

    /// Returns the amount of the LinearWithdrawCapability can mint now.
    public fun withdraw_amount_of_linear_cap<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>): u128 {
        let now = timestamp::now_seconds();
        let elapsed_time = now - cap.start_time;
        if (elapsed_time >= cap.period) {
            cap.total - cap.withdraw
        } else {
            math128::mul_div(cap.total, (elapsed_time as u128), (cap.period as u128)) - cap.withdraw
        }
    }


    /// Check if the given `LinearWithdrawCapability` is empty.
    public fun is_empty_linear_withdraw_cap<TokenT: store>(key: &LinearWithdrawCapability<TokenT>): bool {
        key.total == key.withdraw
    }

    /// Remove mint capability from `signer`.
    public fun remove_withdraw_capability<TokenT: store>(
        signer: &signer
    ): WithdrawCapability<TokenT> acquires WithdrawCapability {
        move_from<WithdrawCapability<TokenT>>(signer::address_of(signer))
    }


    /// Save mint capability to `signer`.
    public fun add_withdraw_capability<TokenT: store>(signer: &signer, cap: WithdrawCapability<TokenT>) {
        move_to(signer, cap)
    }


    /// Destroy the given mint capability.
    public fun destroy_withdraw_capability<TokenT: store>(cap: WithdrawCapability<TokenT>) {
        let WithdrawCapability<TokenT> {} = cap;
    }


    /// Add LinearWithdrawCapability to `signer`, a address only can have one LinearWithdrawCapability<T>
    public fun add_linear_withdraw_capability<TokenT: store>(signer: &signer, cap: LinearWithdrawCapability<TokenT>) {
        move_to(signer, cap)
    }


    /// Remove LinearWithdrawCapability from `signer`.
    public fun remove_linear_withdraw_capability<TokenT: store>(
        signer: &signer
    ): LinearWithdrawCapability<TokenT> acquires LinearWithdrawCapability {
        move_from<LinearWithdrawCapability<TokenT>>(signer::address_of(signer))
    }

    /// Destroy LinearWithdrawCapability.
    public fun destroy_linear_withdraw_capability<TokenT: store>(cap: LinearWithdrawCapability<TokenT>) {
        let LinearWithdrawCapability { total: _, withdraw: _, start_time: _, period: _ } = cap;
    }

    public fun is_empty_linear_withdraw_capability<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>): bool {
        cap.total == cap.withdraw
    }

    /// Get LinearWithdrawCapability total amount
    public fun get_linear_withdraw_capability_total<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>): u128 {
        cap.total
    }

    /// Get LinearWithdrawCapability withdraw amount
    public fun get_linear_withdraw_capability_withdraw<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>): u128 {
        cap.withdraw
    }

    /// Get LinearWithdrawCapability period in seconds
    public fun get_linear_withdraw_capability_period<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>): u64 {
        cap.period
    }

    /// Get LinearWithdrawCapability start_time in seconds
    public fun get_linear_withdraw_capability_start_time<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>): u64 {
        cap.start_time
    }
}