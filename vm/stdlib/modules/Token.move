address 0x1 {
module Token {
    use 0x1::Event;
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::LCS;
    use 0x1::Errors;
    use 0x1::Timestamp;
    use 0x1::Math;

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

    // A fixed time mint key which can mint token until global time > end_time
    resource struct FixedTimeMintKey<TokenType> { total: u128, end_time: u64 }

    // A linear time mint key which can mint token in a peroid by time-based linear release.
    resource struct LinearTimeMintKey<TokenType> { total: u128, minted: u128, start_time: u64, peroid: u64 }

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
        /// The scaling factor for the coin (i.e. the amount to divide by
        /// to get to the human-readable representation for this currency).
        /// e.g. 10^6 for `Coin1`
        scaling_factor: u128,
        /// event stream for minting
        mint_events: Event::EventHandle<MintEvent>,
        /// event stream for burning
        burn_events: Event::EventHandle<BurnEvent>,
    }

    const EDESTORY_TOKEN_NON_ZERO: u64 = 16;
    const EINVALID_ARGUMENT: u64 = 18;
    /// Token register's address should same as TokenType's address.
    const ETOKEN_REGISTER: u64 = 101;

    const EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 102;
    // Mint key time limit
    const EMINT_KEY_TIME_LIMIT: u64 = 103;

    const EDESTROY_KEY_NOT_EMPTY: u64 = 104;
    const EPRECISION_TOO_LARGE: u64 = 105;

    /// 2^128 < 10**39
    const MAX_PRECISION: u8 = 38;

    /// Register the type `TokenType` as a Token and got MintCapability and BurnCapability.
    public fun register_token<TokenType>(
        account: &signer,
        precision: u8,
    ) {
        assert(precision <= MAX_PRECISION, Errors::invalid_argument(EPRECISION_TOO_LARGE));
        let scaling_factor = Math::pow(10, (precision as u64));
        let token_address = token_address<TokenType>();
        assert(Signer::address_of(account) == token_address, Errors::requires_address(ETOKEN_REGISTER));
        move_to(account, MintCapability<TokenType> {});
        move_to(account, BurnCapability<TokenType> {});
        move_to(
            account,
            TokenInfo<TokenType> {
                total_value: 0,
                scaling_factor,
                mint_events: Event::new_event_handle<MintEvent>(account),
                burn_events: Event::new_event_handle<BurnEvent>(account),
            },
        );
    }

    spec fun register_token {
        pragma verify = false; // missing spec for Math::pow
        aborts_if precision > MAX_PRECISION;
        aborts_if Signer::spec_address_of(account) != SPEC_TOKEN_TEST_ADDRESS();
        aborts_if exists<MintCapability<TokenType>>(Signer::spec_address_of(account));
        aborts_if exists<BurnCapability<TokenType>>(Signer::spec_address_of(account));
        aborts_if exists<TokenInfo<TokenType>>(Signer::spec_address_of(account));
        ensures exists<MintCapability<TokenType>>(Signer::spec_address_of(account));
        ensures exists<BurnCapability<TokenType>>(Signer::spec_address_of(account));
        ensures exists<TokenInfo<TokenType>>(Signer::spec_address_of(account));
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
        aborts_if spec_abstract_total_value<TokenType>() + amount > MAX_U128;
        aborts_if !exists<MintCapability<TokenType>>(Signer::address_of(account));
    }

    /// Mint a new Token::Token worth `amount`.
    /// The caller must have a reference to a MintCapability.
    /// Only the Association account can acquire such a reference, and it can do so only via
    /// `borrow_sender_mint_capability`
    public fun mint_with_capability<TokenType>(
        _capability: &MintCapability<TokenType>,
        amount: u128,
    ): Token<TokenType> acquires TokenInfo {
        do_mint(amount)
    }

    spec fun mint_with_capability {
        aborts_if spec_abstract_total_value<TokenType>() + amount > MAX_U128;
        ensures spec_abstract_total_value<TokenType>() ==
                old(global<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS()).total_value) + amount;
    }

    fun do_mint<TokenType>(amount: u128): Token<TokenType> acquires TokenInfo {
        // update market cap resource to reflect minting
        let (token_address, module_name, token_name) = name_of_token<TokenType>();
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        info.total_value = info.total_value + amount;
        Event::emit_event(
            &mut info.mint_events,
            MintEvent {
                amount,
                token_code: code_to_bytes(token_address, module_name, token_name),
            },
        );
        Token<TokenType> { value: amount }
    }

    spec fun do_mint {
        aborts_if !exists<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS());
        aborts_if spec_abstract_total_value<TokenType>() + amount > MAX_U128;
    }

    public fun issue_fixed_mint_key<TokenType>( _capability: &MintCapability<TokenType>,
                                     amount: u128, peroid: u64): FixedTimeMintKey<TokenType>{
        assert(peroid > 0, Errors::invalid_argument(EINVALID_ARGUMENT));
        assert(amount > 0, Errors::invalid_argument(EINVALID_ARGUMENT));
        let now = Timestamp::now_seconds();
        let end_time = now + peroid;
        FixedTimeMintKey{
            total: amount,
            end_time,
        }
    }

    spec fun issue_fixed_mint_key {
        aborts_if peroid == 0;
        aborts_if amount == 0;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(0x1::CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if Timestamp::spec_now_seconds() + peroid > MAX_U64;
    }

    public fun issue_linear_mint_key<TokenType>( _capability: &MintCapability<TokenType>,
                                                amount: u128, peroid: u64): LinearTimeMintKey<TokenType>{
        assert(peroid > 0, Errors::invalid_argument(EINVALID_ARGUMENT));
        assert(amount > 0, Errors::invalid_argument(EINVALID_ARGUMENT));
        let start_time = Timestamp::now_seconds();
        LinearTimeMintKey<TokenType> {
            total: amount,
            minted: 0,
            start_time,
            peroid
        }
    }

    spec fun issue_linear_mint_key {
        aborts_if peroid == 0;
        aborts_if amount == 0;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(0x1::CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun mint_with_fixed_key<TokenType>(key: FixedTimeMintKey<TokenType>): Token<TokenType> acquires TokenInfo {
        let amount = mint_amount_of_fixed_key(&key);
        assert(amount > 0, Errors::invalid_argument(EMINT_KEY_TIME_LIMIT));
        let FixedTimeMintKey { total, end_time:_} = key;
        do_mint(total)
    }

    spec fun mint_with_fixed_key {
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(0x1::CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if spec_mint_amount_of_fixed_key<TokenType>(key) == 0;
        aborts_if !exists<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS());
        aborts_if spec_abstract_total_value<TokenType>() + key.total > MAX_U128;
    }

    public fun mint_with_linear_key<TokenType>(key: &mut LinearTimeMintKey<TokenType>): Token<TokenType> acquires TokenInfo {
        let amount = mint_amount_of_linear_key(key);
        assert(amount > 0, Errors::invalid_argument(EMINT_KEY_TIME_LIMIT));
        let token = do_mint(amount);
        key.minted = key.minted + amount;
        token
    }

    spec fun mint_with_linear_key {
        pragma verify = false; //timeout, fix later
    }

    // Returns the amount of the LinearTimeMintKey can mint now.
    public fun mint_amount_of_linear_key<TokenType>(key: &LinearTimeMintKey<TokenType>): u128 {
        let now = Timestamp::now_seconds();
        let elapsed_time = now - key.start_time;
        if (elapsed_time >= key.peroid) {
            key.total - key.minted
        }else {
            Math::mul_div(key.total, (elapsed_time as u128), (key.peroid as u128)) - key.minted
        }
    }

    spec fun mint_amount_of_linear_key {
        pragma verify = false; // timeout, fix later
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(0x1::CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if Timestamp::spec_now_seconds() < key.start_time;
        aborts_if Timestamp::spec_now_seconds() - key.start_time >= key.peroid && key.total < key.minted;
        aborts_if Timestamp::spec_now_seconds() - key.start_time < key.peroid && Math::spec_mul_div() < key.minted;
    }

    // Returns the mint amount of the FixedTimeMintKey.
    public fun mint_amount_of_fixed_key<TokenType>(key: &FixedTimeMintKey<TokenType>): u128 {
        let now = Timestamp::now_seconds();
        if (now >= key.end_time) {
            key.total
        }else{
            0
        }
    }

    spec fun mint_amount_of_fixed_key {
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(0x1::CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    spec define spec_mint_amount_of_fixed_key<TokenType>(key: FixedTimeMintKey<TokenType>): u128 {
        if (Timestamp::spec_now_seconds() >= key.end_time) {
            key.total
        }else{
            0
        }
    }

    public fun end_time_of_key<TokenType>(key: &FixedTimeMintKey<TokenType>): u64 {
        key.end_time
    }

    public fun destroy_empty_key<TokenType>(key: LinearTimeMintKey<TokenType>) {
        let LinearTimeMintKey<TokenType> { total, minted, start_time: _, peroid: _ } = key;
        assert(total == minted, Errors::invalid_argument(EDESTROY_KEY_NOT_EMPTY));
    }

    spec fun destroy_empty_key {
        aborts_if key.total != key.minted;
    }

    public fun burn<TokenType>(account: &signer, tokens: Token<TokenType>)
    acquires TokenInfo, BurnCapability {
        burn_with_capability(
            borrow_global<BurnCapability<TokenType>>(Signer::address_of(account)),
            tokens,
        )
    }

    spec fun burn {
        aborts_if spec_abstract_total_value<TokenType>() - tokens.value < 0;
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
        aborts_if spec_abstract_total_value<TokenType>() - tokens.value < 0;
        ensures spec_abstract_total_value<TokenType>() ==
                old(global<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS()).total_value) - tokens.value;
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
        aborts_if false;
    }

    /// Splits the given token into two and returns them both
    public fun split<TokenType>(
        token: Token<TokenType>,
        value: u128,
    ): (Token<TokenType>, Token<TokenType>) {
        let other = withdraw(&mut token, value);
        (token, other)
    }

    spec fun split {
        aborts_if token.value < value;
        ensures old(token.value) == result_1.value + result_2.value;
    }

    /// "Divides" the given token into two, where the original token is modified in place.
    /// The original token will have value = original value - `value`
    /// The new token will have a value = `value`
    /// Fails if the tokens value is less than `value`
    public fun withdraw<TokenType>(
        token: &mut Token<TokenType>,
        value: u128,
    ): Token<TokenType> {
        // Check that `value` is less than the token's value
        assert(token.value >= value, Errors::limit_exceeded(EAMOUNT_EXCEEDS_COIN_VALUE));
        token.value = token.value - value;
        Token { value: value }
    }

    spec fun withdraw {
        aborts_if token.value < value;
        ensures result.value == value;
        ensures token.value == old(token).value - value;
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
        assert(value == 0, Errors::invalid_state(EDESTORY_TOKEN_NON_ZERO))
    }

    spec fun destroy_zero {
        aborts_if token.value > 0;
    }

    /// Returns the scaling_factor for the `TokenType` token.
    public fun scaling_factor<TokenType>(): u128 acquires TokenInfo {
        let token_address = token_address<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).scaling_factor
    }

    spec fun scaling_factor {
        aborts_if false;
    }

    /// Return the total amount of token of type `TokenType`.
    public fun market_cap<TokenType>(): u128 acquires TokenInfo {
        let token_address = token_address<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).total_value
    }

    spec fun market_cap {
        aborts_if false;
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
        ensures [abstract] result == SPEC_TOKEN_TEST_ADDRESS();
        ensures [abstract] global<TokenInfo<TokenType>>(result).total_value == 100000000u128;
}

    /// Return the token code for the registered token.
    public fun token_code<TokenType>(): vector<u8> {
        let (addr, module_name, name) = name_of<TokenType>();
        code_to_bytes(addr, module_name, name)
    }

    spec fun token_code {
        pragma opaque = true;
        aborts_if false;
        ensures [abstract] result == spec_token_code<TokenType>();
    }

    /// We use an uninterpreted function to represent the result of derived address. The actual value
    /// does not matter for the verification of callers.
    spec define spec_token_code<TokenType>(): vector<u8>;

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
        ensures [abstract] result_1 == SPEC_TOKEN_TEST_ADDRESS();
        ensures [abstract] global<TokenInfo<TokenType>>(result_1).total_value == 100000000u128;
    }

    spec module {
        define SPEC_TOKEN_TEST_ADDRESS(): address {
            0x2
        }

        define spec_abstract_total_value<TokenType>(): u128 {
            global<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS()).total_value
        }

    }

}
}