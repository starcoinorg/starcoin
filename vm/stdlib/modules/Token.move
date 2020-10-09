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

    resource struct ScalingFactorModifyCapability<TokenType> { }

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
        base_scaling_factor: u128,
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
    const ETOKEN_REGISTER: u64 = 100;
    // TokenType's name should same as Token's Module name.
    // const ETOKEN_NAME: u64 = 101;
    const EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 102;

    /// Register the type `TokenType` as a Token and got MintCapability and BurnCapability.
    public fun register_token<TokenType>(
        account: &signer,
        base_scaling_factor: u128,
        fractional_part: u128,
    ) {
        let (token_address, _module_name, _token_name) = name_of<TokenType>();
        assert(Signer::address_of(account) == token_address, ETOKEN_REGISTER);
        // assert(module_name == token_name, ETOKEN_NAME);
        move_to(account, MintCapability<TokenType> {});
        move_to(account, BurnCapability<TokenType> {});
        move_to(account, ScalingFactorModifyCapability<TokenType> {});
        move_to(
            account,
            TokenInfo<TokenType> {
                total_value: 0,
                scaling_factor: base_scaling_factor,
                base_scaling_factor,
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

    public fun remove_scaling_factor_modify_capability<TokenType>(
        signer: &signer,
    ): ScalingFactorModifyCapability<TokenType> acquires ScalingFactorModifyCapability {
        move_from<ScalingFactorModifyCapability<TokenType>>(Signer::address_of(signer))
    }

    spec fun remove_scaling_factor_modify_capability {
        pragma verify = false;
    }

    public fun add_scaling_factor_modify_capability<TokenType>(
        signer: &signer,
        cap: ScalingFactorModifyCapability<TokenType>,
    ) {
        move_to<ScalingFactorModifyCapability<TokenType>>(signer, cap)
    }

    spec fun add_scaling_factor_modify_capability {
        pragma verify = false;
    }

    public fun destroy_scaling_factor_modify_capability<TokenType>(
        cap: ScalingFactorModifyCapability<TokenType>,
    ) {
        let ScalingFactorModifyCapability<TokenType> { } = cap;
    }

    spec fun destroy_scaling_factor_modify_capability {
        pragma verify = false;
    }

    public fun remove_mint_capability<TokenType>(signer: &signer): MintCapability<TokenType>
    acquires MintCapability {
        move_from<MintCapability<TokenType>>(Signer::address_of(signer))
    }

    spec fun remove_mint_capability {
        aborts_if !exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures !exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun add_mint_capability<TokenType>(signer: &signer, cap: MintCapability<TokenType>) {
        move_to(signer, cap)
    }

    spec fun add_mint_capability {
        aborts_if exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures exists<MintCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun destroy_mint_capability<TokenType>(cap: MintCapability<TokenType>) {
        let MintCapability<TokenType> { } = cap;
    }

    spec fun destroy_mint_capability {
    }

    public fun remove_burn_capability<TokenType>(signer: &signer): BurnCapability<TokenType>
    acquires BurnCapability {
        move_from<BurnCapability<TokenType>>(Signer::address_of(signer))
    }

    spec fun remove_burn_capability {
        aborts_if !exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures !exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun add_burn_capability<TokenType>(signer: &signer, cap: BurnCapability<TokenType>) {
        move_to(signer, cap)
    }

    spec fun add_burn_capability {
        aborts_if exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
        ensures exists<BurnCapability<TokenType>>(Signer::spec_address_of(signer));
    }

    public fun destroy_burn_capability<TokenType>(cap: BurnCapability<TokenType>) {
        let BurnCapability<TokenType> { } = cap;
    }

    spec fun destroy_burn_capability {
    }

    /// Return `amount` tokens.
    /// Fails if the sender does not have a published MintCapability.
    public fun mint<TokenType>(account: &signer, amount: u128): Token<TokenType>
    acquires TokenInfo, MintCapability {
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

    /// Mint a new Token::Token worth `amount` considering current `scaling_factor`. The caller must have a reference to a MintCapability.
    /// Only the Association account can acquire such a reference, and it can do so only via
    /// `borrow_sender_mint_capability`
    public fun mint_with_capability<TokenType>(
        _capability: &MintCapability<TokenType>,
        amount: u128,
    ): Token<TokenType> acquires TokenInfo {
        // update market cap resource to reflect minting
        let (token_address, module_name, token_name) = name_of_token<TokenType>();
        let share = amount_to_share<TokenType>(amount);
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        info.total_value = info.total_value + (share as u128);
        Event::emit_event(
            &mut info.mint_events,
            MintEvent {
                amount: share,
                token_code: code_to_bytes(token_address, module_name, token_name),
            },
        );
        Token<TokenType> { value: share }
    }

    spec fun mint_with_capability {
        pragma verify = false;
        //Todo: fix name_of()
    }

    public fun burn<TokenType>(account: &signer, tokens: Token<TokenType>)
    acquires TokenInfo, BurnCapability {
        burn_with_capability(
            borrow_global<BurnCapability<TokenType>>(Signer::address_of(account)),
            tokens,
        )
    }

    spec fun burn {
        aborts_if !exists<BurnCapability<TokenType>>(Signer::spec_address_of(account));
    }

    public fun burn_with_capability<TokenType>(
        _capability: &BurnCapability<TokenType>,
        tokens: Token<TokenType>,
    ) acquires TokenInfo {
        let (token_address, module_name, token_name) = name_of_token<TokenType>();
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        let Token { value } = tokens;
        info.total_value = info.total_value - value;
        Event::emit_event(
            &mut info.burn_events,
            BurnEvent {
                amount: value,
                token_code: code_to_bytes(token_address, module_name, token_name),
            },
        );
    }

    spec fun burn_with_capability {
        aborts_if false;
    }

    /// Create a new Token::Token<TokenType> with a value of 0
    public fun zero<TokenType>(): Token<TokenType> {
        Token<TokenType> { value: 0 }
    }

    spec fun zero {
    }

    /// Scaled value of the token considering the `scaling_factor`.
    public fun value<TokenType>(token: &Token<TokenType>): u128 acquires TokenInfo {
        share_to_amount<TokenType>(share(token))
    }

    spec fun value {
        pragma verify = false;
    }

    /// Public accessor for the value of a token
    public fun share<TokenType>(token: &Token<TokenType>): u128 {
        token.value
    }

    /// Splits the given token into two and returns them both
    /// It leverages `Self::split_share` for any verifications of the values
    public fun split<TokenType>(
        token: Token<TokenType>,
        amount: u128,
    ): (Token<TokenType>, Token<TokenType>) acquires TokenInfo {
        split_share<TokenType>(token, amount_to_share<TokenType>(amount))
    }

    spec fun split {
        pragma verify = false;
    }

    /// Splits the given token into two and returns them both
    /// It leverages `Self::withdraw_share` for any verifications of the values.
    /// It operates on token value directly regardless of the `scaling_factor` of the token.
    public fun split_share<TokenType>(
        token: Token<TokenType>,
        share: u128,
    ): (Token<TokenType>, Token<TokenType>) {
        let other = withdraw_share(&mut token, share);
        (token, other)
    }

    spec fun split_share {
        aborts_if token.value < share;
        // TODO: ensure result
    }

    /// "Divides" the given token into two, where the original token is modified in place.
    /// This will consider the scaling_factor of the `Token`.
    public fun withdraw<TokenType>(token: &mut Token<TokenType>, amount: u128): Token<TokenType>
    acquires TokenInfo {
        withdraw_share<TokenType>(token, amount_to_share<TokenType>(amount))
    }

    // spec fun withdraw {
    //     aborts_if token.value < amount;
    //     ensures result.value == amount;
    //     ensures token.value == old(token).value - amount;
    // }
    spec fun withdraw {
        pragma verify = false;
    }

    /// It operates on token value directly regardless of the `scaling_factor` of the token.
    /// The original token will have value = original value - `share`
    /// The new token will have a value = `share`
    /// Fails if the tokens value is less than `share`
    public fun withdraw_share<TokenType>(
        token: &mut Token<TokenType>,
        share: u128,
    ): Token<TokenType> {
        // Check that `share` is less than the token's value
        assert(token.value >= share, EAMOUNT_EXCEEDS_COIN_VALUE);
        token.value = token.value - share;
        Token { value: share }
    }

    spec fun withdraw_share {
        aborts_if token.value < share;
        ensures result.value == share;
        ensures token.value == old(token).value - share;
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
        let Token { value } = check;
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
        let Token { value } = token;
        assert(value == 0, ErrorCode::EDESTORY_TOKEN_NON_ZERO())
    }

    spec fun destroy_zero {
        aborts_if token.value > 0;
    }

    /// convenient function to calculate hold of the input `amount` based the current scaling_factor.
    public fun amount_to_share<TokenType>(amount: u128): u128 acquires TokenInfo {
        let base = base_scaling_factor<TokenType>();
        let scaled = scaling_factor<TokenType>();
        // shortcut to avoid bignumber cal.
        if (base == scaled) {
            amount
        } else {
            amount * base / scaled
        }
    }

    spec fun amount_to_share {
        pragma verify = false;
    }

    public fun share_to_amount<TokenType>(hold: u128): u128 acquires TokenInfo {
        let base = base_scaling_factor<TokenType>();
        let scaled = scaling_factor<TokenType>();
        if (base == scaled) {
            hold
        } else {
            hold * scaled / base
        }
    }

    spec fun share_to_amount {
        pragma verify = false;
    }

    /// Returns the scaling factor for the `TokenType` token.
    public fun scaling_factor<TokenType>(): u128 acquires TokenInfo {
        let (token_address, _, _) = name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).scaling_factor
    }

    spec fun scaling_factor {
        pragma verify = false;
    }

    public fun base_scaling_factor<TokenType>(): u128 acquires TokenInfo {
        let (token_address, _, _) = name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).base_scaling_factor
    }

    spec fun base_scaling_factor {
        pragma verify = false;
    }

    public fun set_scaling_factor<TokenType>(signer: &signer, value: u128)
    acquires TokenInfo, ScalingFactorModifyCapability {
        let cap = borrow_global<ScalingFactorModifyCapability<TokenType>>(
            Signer::address_of(signer),
        );
        set_scaling_factor_with_capability(cap, value)
    }

    spec fun set_scaling_factor {
        pragma verify = false;
    }

    public fun set_scaling_factor_with_capability<TokenType>(
        _cap: &ScalingFactorModifyCapability<TokenType>,
        value: u128,
    ) acquires TokenInfo {
        let token_address = token_address<TokenType>();
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        info.scaling_factor = value;

        // TODO: emit event
    }

    spec fun set_scaling_factor_with_capability {
        aborts_if false;
    }

    /// Returns the representable fractional part for the `TokenType` token.
    public fun fractional_part<TokenType>(): u128 acquires TokenInfo {
        let token_address = token_address<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).fractional_part
    }

    spec fun fractional_part {
        aborts_if false;
    }

    /// Return the total amount of token of type `TokenType` considering current `scaling_factor`
    public fun market_cap<TokenType>(): u128 acquires TokenInfo {
        share_to_amount<TokenType>(total_share<TokenType>())
    }

    spec fun market_cap {
        // Todo: fix name_of()
        pragma verify = false;
        //aborts_if !exists<TokenInfo<TokenType>>(token_module_address());
    }

    /// Return the total share of token minted.
    public fun total_share<TokenType>(): u128 acquires TokenInfo {
        let (token_address, _, _) = name_of<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).total_value
    }

    spec fun total_share {
        pragma verify = false;
    }

    /// Return true if the type `TokenType` is a registered in `token_address`.
    public fun is_registered_in<TokenType>(token_address: address): bool {
        exists<TokenInfo<TokenType>>(token_address)
    }

    spec fun is_registered_in {
        aborts_if false;
    }

    /// Return true if the type `TokenType1` is same with `TokenType2`
    public fun is_same_token<TokenType1, TokenType2>(): bool {
        return token_code<TokenType1>() == token_code<TokenType2>()
    }

    spec fun is_same_token {
        aborts_if false;
    }

    /// Return the TokenType's address
    public fun token_address<TokenType>(): address {
        let (addr, _, _) = name_of<TokenType>();
        addr
    }

    // The specification of this function is abstracted to avoid the complexity to
    // return a real address to caller
    spec fun token_address {
        pragma opaque = true;
        aborts_if false;
        ensures [abstract] exists<TokenInfo<TokenType>>(result);
}

    /// Return the token code for the registered token.
    public fun token_code<TokenType>(): vector<u8> {
        let (addr, module_name, name) = name_of<TokenType>();
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
        pragma opaque = true;
        aborts_if false;
    }

    fun name_of_token<TokenType>(): (address, vector<u8>, vector<u8>) {
        name_of<TokenType>()
    }

    // The specification of this function is abstracted to avoid the complexity to
    // return a real address to caller
    spec fun name_of_token {
        pragma opaque = true;
        aborts_if false;
        ensures [abstract] exists<TokenInfo<TokenType>>(result_1);
        ensures [abstract] global<TokenInfo<TokenType>>(result_1).total_value == MAX_U128;
}
}
}