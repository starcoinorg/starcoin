address 0x1 {
module Token {
    use 0x1::Event;
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::LCS;
    use 0x1::Generic::type_of as name_of;
    /// The currency has a `TokenType` color that tells us what currency the
    /// `value` inside represents.
    resource struct Coin<TokenType> {
        value: u64,
    }

    /// A minting capability allows coins of type `TokenType` to be minted
    resource struct MintCapability<TokenType> { }

    resource struct BurnCapability<TokenType> { }

    struct MintEvent {
        /// funds added to the system
        amount: u64,
        /// full info of Token.
        token_code: vector<u8>,
    }

    struct BurnEvent {
        /// funds removed from the system
        amount: u64,
        /// full info of Token
        token_code: vector<u8>,
    }

    resource struct TokenInfo<TokenType> {
        /// The total value for the currency represented by
        /// `TokenType`. Mutable.
        total_value: u128,
        /// The scaling factor for the coin (i.e. the amount to multiply by
        /// to get to the human-readable reprentation for this currency). e.g. 10^6 for Coin1
        scaling_factor: u64,
        /// The smallest fractional part (number of decimal places) to be
        /// used in the human-readable representation for the currency (e.g.
        /// 10^2 for Coin1 cents)
        fractional_part: u64,
        // The code symbol for this `TokenType`. UTF-8 encoded.
        // e.g. for "STC" this is x"4C4252". No character limit.
        // token_code: TokenCode,
        /// event stream for minting
        mint_events: Event::EventHandle<MintEvent>,
        /// event stream for burning
        burn_events: Event::EventHandle<BurnEvent>,
    }

    /// Register the type `TokenType` as a currency. Without this, a type
    /// cannot be used as a coin/currency unit n Libra.
    public fun register_currency<TokenType>(
        account: &signer,
        scaling_factor: u64,
        fractional_part: u64,
    ) {
        let (token_address, _, _) = name_of<TokenType>();
        assert(Signer::address_of(account) == token_address, 401);
        move_to(account, MintCapability<TokenType> {});
        move_to(account, BurnCapability<TokenType> {});
        move_to(
            account,
            TokenInfo<TokenType> {
                total_value: 0,
                scaling_factor,
                fractional_part,
                mint_events: Event::new_event_handle<MintEvent>(account),
                burn_events: Event::new_event_handle<BurnEvent>(account),
            },
        );
    }

    public fun remove_mint_capability<TokenType>(
        signer: &signer,
    ): MintCapability<TokenType> acquires MintCapability {
        move_from<MintCapability<TokenType>>(Signer::address_of(signer))
    }

    public fun add_mint_capability<TokenType>(signer: &signer,
    cap: MintCapability<TokenType>)  {
        move_to(signer, cap)
    }

    public fun destroy_mint_capability<TokenType>(cap: MintCapability<TokenType>) {
        let MintCapability<TokenType>{  } = cap;
    }


    public fun remove_burn_capability<TokenType>(
        signer: &signer,
    ): BurnCapability<TokenType> acquires BurnCapability {
        move_from<BurnCapability<TokenType>>(Signer::address_of(signer))
    }

    public fun add_burn_capability<TokenType>(signer: &signer,
        cap: BurnCapability<TokenType>)  {
            move_to(signer, cap)
        }


    public fun destroy_burn_capability<TokenType>(cap: BurnCapability<TokenType>) {
        let BurnCapability<TokenType>{  } = cap;
    }

    /// Return `amount` coins.
    /// Fails if the sender does not have a published MintCapability.
    public fun mint<TokenType>(
        account: &signer,
        amount: u64,
    ): Coin<TokenType> acquires TokenInfo, MintCapability {
        mint_with_capability(
            borrow_global<MintCapability<TokenType>>(Signer::address_of(account)),
            amount,
        )
    }

    /// Mint a new Coin::Coin worth `value`. The caller must have a reference to a MintCapability.
    /// Only the Association account can acquire such a reference, and it can do so only via
    /// `borrow_sender_mint_capability`
    public fun mint_with_capability<TokenType>(
        _capability: &MintCapability<TokenType>,
        value: u64,
    ): Coin<TokenType> acquires TokenInfo {
        // update market cap resource to reflect minting
        // assert_is_token<TokenType>();
        let (token_address, module_name, token_name) = name_of<TokenType>();
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        info.total_value = info.total_value + (value as u128);
        Event::emit_event(
            &mut info.mint_events,
            MintEvent {
                amount: value,
                token_code: code_to_bytes(token_address, module_name, token_name),
            }
        );
        Coin<TokenType> { value }
    }

    public fun burn<TokenType>(
        account: &signer,
        tokens: Coin<TokenType>,
    ) acquires TokenInfo, BurnCapability {
        burn_with_capability(
            borrow_global<BurnCapability<TokenType>>(Signer::address_of(account)),
            tokens,
        )
    }

    public fun burn_with_capability<TokenType>(
        _capability: &BurnCapability<TokenType>,
        tokens: Coin<TokenType>,
    ) acquires TokenInfo {
        let (token_address, module_name, token_name) = name_of<TokenType>();
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        let Coin{ value: value } = tokens;
        info.total_value = info.total_value - (value as u128);
        Event::emit_event(
            &mut info.burn_events,
            BurnEvent {
                amount: value,
                token_code: code_to_bytes(token_address, module_name, token_name),
            }
        );

    }

    /// Create a new Coin::Coin<TokenType> with a value of 0
    public fun zero<TokenType>(): Coin<TokenType> {
        Coin<TokenType> { value: 0 }
    }

    /// Public accessor for the value of a coin
    public fun value<TokenType>(coin: &Coin<TokenType>): u64 {
        coin.value
    }

    /// Splits the given coin into two and returns them both
    /// It leverages `Self::withdraw` for any verifications of the values
    public fun split<TokenType>(
        coin: Coin<TokenType>,
        amount: u64,
    ): (Coin<TokenType>, Coin<TokenType>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    /// "Divides" the given coin into two, where the original coin is modified in place
    /// The original coin will have value = original value - `amount`
    /// The new coin will have a value = `amount`
    /// Fails if the coins value is less than `amount`
    public fun withdraw<TokenType>(
        coin: &mut Coin<TokenType>,
        amount: u64,
    ): Coin<TokenType> {
        // Check that `amount` is less than the coin's value
        assert(coin.value >= amount, 10);
        coin.value = coin.value - amount;
        Coin { value: amount }
    }

    /// Merges two coins of the same currency and returns a new coin whose
    /// value is equal to the sum of the two inputs
    public fun join<TokenType>(
        coin1: Coin<TokenType>,
        coin2: Coin<TokenType>,
    ): Coin<TokenType> {
        deposit(&mut coin1, coin2);
        coin1
    }

    /// "Merges" the two coins
    /// The coin passed in by reference will have a value equal to the sum of the two coins
    /// The `check` coin is consumed in the process
    public fun deposit<TokenType>(coin: &mut Coin<TokenType>, check: Coin<TokenType>) {
        let Coin{ value: value } = check;
        coin.value = coin.value + value;
    }

    /// Destroy a coin
    /// Fails if the value is non-zero
    /// The amount of Coin in the system is a tightly controlled property,
    /// so you cannot "burn" any non-zero amount of Coin
    public fun destroy_zero<TokenType>(coin: Coin<TokenType>) {
        let Coin{ value: value } = coin;
        assert(value == 0, 5)
    }


    /// Returns the scaling factor for the `CoinType` currency.
    public fun scaling_factor<TokenType>(): u64
    acquires TokenInfo {
        let (token_address, _, _) =name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).scaling_factor
    }

    /// Returns the representable fractional part for the `CoinType` currency.
    public fun fractional_part<TokenType>(): u64
    acquires TokenInfo {
        let (token_address, _, _) =name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).fractional_part
    }



    /// Return the total amount of currency minted of type `TokenType`
    public fun market_cap<TokenType>(): u128 acquires TokenInfo {
        let (token_address, _, _) =name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).total_value
    }

    /// Return true if the type `TokenType` is a registered in `token_address`.
    public fun is_registered_in<TokenType>(token_address: address): bool {
        exists<TokenInfo<TokenType>>(token_address)
    }

    /// Return the token code for the registered token.
    public fun token_code<TokenType>(): vector<u8> {
        let (addr, module_name, name) =name_of<TokenType>();
        code_to_bytes(addr, module_name, name)
    }

    fun code_to_bytes(addr: address, module_name: vector<u8>, name: vector<u8>): vector<u8> {
        let code = LCS::to_bytes(&addr);

        // {{addr}}::{{module}}::{{struct}}
        Vector::append(&mut code, b"::");
        Vector::append(&mut code, module_name);
        Vector::append(&mut code, b"::");
        Vector::append(&mut code, name);

        code
    }

    public fun assert_is_token<TokenType>() {
        assert(is_token<TokenType>(), 400);
    }

    public fun is_token<TokenType>(): bool {
        let (addr, _module_name, _name) =name_of<TokenType>();
        is_registered_in<TokenType>(addr)
    }

    // /// Native method to get struct's:
    // /// - address
    // /// - module_name
    // /// - struct_name
    // native fun name_of<TokenType>(): (address, vector<u8>, vector<u8>);
}
}