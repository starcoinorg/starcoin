address 0x1 {
module StableCoin {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Balance;
    use 0x1::Option;

    // const token_address(): address = 0x1;
    resource struct T { }

    resource struct Setting {
        frozen: bool,
        max_withdraw_amount: Option::Option<u64>,
    }

    resource struct SharedCapability {
        withdraw_cap: Balance::WithdrawCapability<T>,
    }

    public fun token_address(): address {
        0x1
    }

    //// Methods for Token Issuer.

    /// Initialize StableCoin.
    public fun initialize(signer: &signer) {
        assert(Signer::address_of(signer) == token_address(), 401);
        let t = T {};
        // register currency.
        Token::register_currency<T>(signer, &t, 1000, 1000);
        // create shared withdraw capability
        let withdraw_cap = Balance::create_withdraw_capability<T>(&t);
        move_to(signer, SharedCapability { withdraw_cap });
        // TODO: for later use
        move_to(signer, t);
    }

    /// Mint Some Coin to `receiver`.
    /// can only be called by address who has `T`.
    public fun mint_to(signer: &signer, amount: u64, receiver: address) {
        let tokens = Token::mint<T>(signer, amount, token_address());
        Balance::deposit_to(receiver, tokens);
    }

    /// burn `amount` coin of my own.
    /// can only be called by address who has `T`.
    public fun burn(signer: &signer, amount: u64) acquires SharedCapability {
        let shared_cap = borrow_global<SharedCapability>(token_address());
        let coins_to_burn = Balance::withdraw_with_capability<T>(
            &shared_cap.withdraw_cap,
            Signer::address_of(signer),
            amount,
        );
        Token::burn(signer, token_address(), coins_to_burn);
    }

    /// Admin can update anyone's setting.
    public fun update_max_withdraw_amount(
        signer: &signer,
        user: address,
        max_withdraw_amount: Option::Option<u64>,
    ) acquires Setting {
        assert(Signer::address_of(signer) == token_address(), 401);
        borrow_global_mut<Setting>(user).max_withdraw_amount = max_withdraw_amount;
    }

    /// Admin can ban user.
    public fun ban_user(signer: &signer, user: address) acquires Setting {
        assert(Signer::address_of(signer) == token_address(), 401);
        borrow_global_mut<Setting>(user).frozen = true;
    }

    /// Admin can unban user.
    public fun defreeze(signer: &signer, user: address) acquires Setting {
        assert(Signer::address_of(signer) == token_address(), 401);
        borrow_global_mut<Setting>(user).frozen = false;
    }

    //// Methods for users of the coin.

    /// User creates Account to participate in the StableCoin.
    public fun accept_coin(signer: &signer, max_withdraw_amount: Option::Option<u64>) {
        Balance::accept_token<T>(signer);
        let user_setting = Setting { max_withdraw_amount, frozen: false };
        move_to(signer, user_setting);
    }

    /// User `signer` transfer `amount` coins to receiver.
    public fun transfer(signer: &signer, receiver: address, amount: u64)
    acquires Setting, SharedCapability {
        let user_setting = borrow_global<Setting>(Signer::address_of(signer));
        // check user is not freezed.
        assert(!user_setting.frozen, 503);
        let max_amount = &user_setting.max_withdraw_amount;
        // check max amount.
        if (Option::is_some(max_amount)) {
            assert(amount <= *Option::borrow(max_amount), 402);
        };
        let shared_cap = borrow_global<SharedCapability>(token_address());
        let coins = Balance::withdraw_with_capability<T>(
            &shared_cap.withdraw_cap,
            Signer::address_of(signer),
            amount,
        );
        Balance::deposit_to<T>(receiver, coins);
    }
}
}