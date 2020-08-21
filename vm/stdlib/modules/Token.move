address 0x1 {
module Token {
    use 0x1::Event;
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::LCS;
    use 0x1::ErrorCode;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    /// The token has a `TokenType` color that tells us what token the
    /// `value` inside represents.
    resource struct Token<TokenType> {
        value: u128,
    }

    /// A minting capability allows tokens of type `TokenType` to be minted
    resource struct MintCapability<TokenType> { }

    resource struct BurnCapability<TokenType> { }

    struct MintEvent {
        /// funds added to the system
        amount: u128,
        /// full info of Token.
        token_code: vector<u8>,
    }

    struct BurnEvent {
        /// funds removed from the system
        amount: u128,
        /// full info of Token
        token_code: vector<u8>,
    }

    resource struct TokenInfo<TokenType> {
        /// The total value for the token represented by
        /// `TokenType`. Mutable.
        total_value: u128,
        /// The scaling factor for the token (i.e. the amount to multiply by
        /// to get to the human-readable reprentation for this token). e.g. 10^6 for Token1
        scaling_factor: u128,
        /// The smallest fractional part (number of decimal places) to be
        /// used in the human-readable representation for the token (e.g.
        /// 10^2 for Token1 cents)
        fractional_part: u128,
        /// event stream for minting
        mint_events: Event::EventHandle<MintEvent>,
        /// event stream for burning
        burn_events: Event::EventHandle<BurnEvent>,
    }

    /// Token register's address should same as TokenType's address.
    const ETOKEN_REGISTER:u64 = 100;
    /// TokenType's name should same as Token's Module name.
    const ETOKEN_NAME:u64 = 101;
    const EAMOUNT_EXCEEDS_COIN_VALUE:u64 = 102;

    /// Register the type `TokenType` as a Token and got MintCapability and BurnCapability.
    public fun register_token<TokenType>(
        account: &signer,
        scaling_factor: u128,
        fractional_part: u128,
    ) {
        let (token_address, module_name, token_name) = name_of<TokenType>();
        assert(Signer::address_of(account) == token_address, ETOKEN_REGISTER);
        assert(module_name == token_name, ETOKEN_NAME);
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

    spec fun register_token {
        // Todo: fix name_of()
        pragma verify = false;
    }

    public fun remove_mint_capability<TokenType>(
        signer: &signer,
    ): MintCapability<TokenType> acquires MintCapability {
        move_from<MintCapability<TokenType>>(Signer::address_of(signer))
    }

    spec fun remove_mint_capability {
        aborts_if !exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures !exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun add_mint_capability<TokenType>(signer: &signer,
    cap: MintCapability<TokenType>)  {
        move_to(signer, cap)
    }

    spec fun add_mint_capability {
        aborts_if exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun destroy_mint_capability<TokenType>(cap: MintCapability<TokenType>) {
        let MintCapability<TokenType>{  } = cap;
    }

    spec fun destroy_mint_capability {
    }

    public fun remove_burn_capability<TokenType>(
        signer: &signer,
    ): BurnCapability<TokenType> acquires BurnCapability {
        move_from<BurnCapability<TokenType>>(Signer::address_of(signer))
    }

    spec fun remove_burn_capability {
        aborts_if !exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures !exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun add_burn_capability<TokenType>(signer: &signer,
        cap: BurnCapability<TokenType>)  {
            move_to(signer, cap)
    }

    spec fun add_burn_capability {
        aborts_if exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun destroy_burn_capability<TokenType>(cap: BurnCapability<TokenType>) {
        let BurnCapability<TokenType>{  } = cap;
    }

    spec fun destroy_burn_capability {
    }

    /// Return `amount` tokens.
    /// Fails if the sender does not have a published MintCapability.
    public fun mint<TokenType>(
        account: &signer,
        amount: u128,
    ): Token<TokenType> acquires TokenInfo, MintCapability {
        mint_with_capability(
            borrow_global<MintCapability<TokenType>>(Signer::address_of(account)),
            amount,
        )
    }

    spec fun mint {
        pragma verify = false;

        aborts_if !exists<MintCapability<TokenType>>(Signer::address_of(account));
        //Todo: fix name_of()
    }

    /// Mint a new Token::Token worth `value`. The caller must have a reference to a MintCapability.
    /// Only the Association account can acquire such a reference, and it can do so only via
    /// `borrow_sender_mint_capability`
    public fun mint_with_capability<TokenType>(
        _capability: &MintCapability<TokenType>,
        value: u128,
    ): Token<TokenType> acquires TokenInfo {
        // update market cap resource to reflect minting
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
        Token<TokenType> { value }
    }

    spec fun mint_with_capability {
        pragma verify = false;

        //Todo: fix name_of()
    }

    public fun burn<TokenType>(
        account: &signer,
        tokens: Token<TokenType>,
    ) acquires TokenInfo, BurnCapability {
        burn_with_capability(
            borrow_global<BurnCapability<TokenType>>(Signer::address_of(account)),
            tokens,
        )
    }

    spec fun burn {
        pragma verify = false;

        //Todo: fix name_of()
        aborts_if !exists<BurnCapability<TokenType>>(Signer::address_of(account));
    }

    public fun burn_with_capability<TokenType>(
        _capability: &BurnCapability<TokenType>,
        tokens: Token<TokenType>,
    ) acquires TokenInfo {
        let (token_address, module_name, token_name) = name_of<TokenType>();
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        let Token{ value: value } = tokens;
        info.total_value = info.total_value - (value as u128);
        Event::emit_event(
            &mut info.burn_events,
            BurnEvent {
                amount: value,
                token_code: code_to_bytes(token_address, module_name, token_name),
            }
        );

    }

    spec fun burn_with_capability {
        pragma verify = false;

        //Todo: fix name_of()
        //aborts_if !exists<TokenInfo<TokenType>>(token_address);
    }

    /// Create a new Token::Token<TokenType> with a value of 0
    public fun zero<TokenType>(): Token<TokenType> {
        Token<TokenType> { value: 0 }
    }

    spec fun zero {
    }

    /// Public accessor for the value of a token
    public fun value<TokenType>(token: &Token<TokenType>): u128 {
        token.value
    }

    spec fun value {
    }

    /// Splits the given token into two and returns them both
    /// It leverages `Self::withdraw` for any verifications of the values
    public fun split<TokenType>(
        token: Token<TokenType>,
        amount: u128,
    ): (Token<TokenType>, Token<TokenType>) {
        let other = withdraw(&mut token, amount);
        (token, other)
    }

    spec fun split {
        aborts_if token.value < amount;
        // TODO: ensure result
    }

    /// "Divides" the given token into two, where the original token is modified in place
    /// The original token will have value = original value - `amount`
    /// The new token will have a value = `amount`
    /// Fails if the tokens value is less than `amount`
    public fun withdraw<TokenType>(
        token: &mut Token<TokenType>,
        amount: u128,
    ): Token<TokenType> {
        // Check that `amount` is less than the token's value
        assert(token.value >= amount, EAMOUNT_EXCEEDS_COIN_VALUE);
        token.value = token.value - amount;
        Token { value: amount }
    }

    spec fun withdraw {
        aborts_if token.value < amount;
        ensures result.value == amount;
        ensures token.value == old(token).value - amount;
    }

    /// Merges two tokens of the same token and returns a new token whose
    /// value is equal to the sum of the two inputs
    public fun join<TokenType>(
        token1: Token<TokenType>,
        token2: Token<TokenType>,
    ): Token<TokenType> {
        deposit(&mut token1, token2);
        token1
    }

    spec fun join {
        aborts_if token1.value + token2.value > max_u128();
        ensures old(token1).value + old(token2).value == result.value;
        ensures token1.value + token2.value == result.value;
    }

    /// "Merges" the two tokens
    /// The token passed in by reference will have a value equal to the sum of the two tokens
    /// The `check` token is consumed in the process
    public fun deposit<TokenType>(token: &mut Token<TokenType>, check: Token<TokenType>) {
        let Token{ value: value } = check;
        token.value = token.value + value;
    }

    spec fun deposit {
        aborts_if token.value + check.value > max_u128();
        ensures old(token).value + check.value == token.value;
    }

    /// Destroy a token
    /// Fails if the value is non-zero
    /// The amount of Token in the system is a tightly controlled property,
    /// so you cannot "burn" any non-zero amount of Token
    public fun destroy_zero<TokenType>(token: Token<TokenType>) {
        let Token{ value: value } = token;
        assert(value == 0, ErrorCode::EDESTORY_TOKEN_NON_ZERO())
    }

    spec fun destroy_zero {
        aborts_if token.value > 0;
    }

    /// Returns the scaling factor for the `TokenType` token.
    public fun scaling_factor<TokenType>(): u128
    acquires TokenInfo {
        let (token_address, _, _) =name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).scaling_factor
    }

    spec fun scaling_factor {
        // Todo: fix name_of()
        pragma verify = false;

        //let x  = name_of();
        //aborts_if !exists<TokenInfo<TokenType>>(x);
    }

    /// Returns the representable fractional part for the `TokenType` token.
    public fun fractional_part<TokenType>(): u128
    acquires TokenInfo {
        let (token_address, _, _) =name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).fractional_part
    }

    spec fun fractional_part {
        // Todo: fix name_of()
        pragma verify = false;
    }

    /// Return the total amount of token minted of type `TokenType`
    public fun market_cap<TokenType>(): u128 acquires TokenInfo {
        let (token_address, _, _) =name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).total_value
    }

    spec fun market_cap {
        // Todo: fix name_of()
        pragma verify = false;
        //aborts_if !exists<TokenInfo<TokenType>>(token_module_address());
    }

    /// Return true if the type `TokenType` is a registered in `token_address`.
    public fun is_registered_in<TokenType>(token_address: address): bool {
        exists<TokenInfo<TokenType>>(token_address)
    }

    spec fun is_registered_in {
        aborts_if false;
    }

    /// Return true if the type `TokenType1` is same with `TokenType2`
    public fun is_same_token<TokenType1,TokenType2>(): bool {
        return token_code<TokenType1>() == token_code<TokenType2>()
    }

    spec fun is_same_token {
        aborts_if false;
    }

    /// Return the TokenType's address
    public fun token_address<TokenType>():address {
        let (addr, _, _) =name_of<TokenType>();
        addr
    }

    spec fun token_address {
        aborts_if false;
    }

    /// Return the token code for the registered token.
    public fun token_code<TokenType>(): vector<u8> {
        let (addr, module_name, name) =name_of<TokenType>();
        code_to_bytes(addr, module_name, name)
    }

    spec fun token_code {
        aborts_if false;
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

    spec fun code_to_bytes {
        aborts_if false;
    }

    /// Return Token's module address, module name, and type name of `TokenType`.
    native fun name_of<TokenType>(): (address, vector<u8>, vector<u8>);

    spec fun name_of {
        pragma intrinsic = true;
    }
}
}