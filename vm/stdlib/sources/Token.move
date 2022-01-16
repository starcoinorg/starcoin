address StarcoinFramework {
/// Token implementation of Starcoin.
module Token {
    use StarcoinFramework::Event;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Math;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict = true;
    }

    /// The token has a `TokenType` color that tells us what token the
    /// `value` inside represents.
    struct Token<phantom TokenType> has store {
        value: u128,
    }

    /// Token Code which identify a unique Token.
    struct TokenCode has copy, drop, store {
        /// address who define the module contains the Token Type.
        addr: address,
        /// module which contains the Token Type.
        module_name: vector<u8>,
        /// name of the token. may nested if the token is a instantiated generic token type.
        name: vector<u8>,
    }

    /// A minting capability allows tokens of type `TokenType` to be minted
    struct MintCapability<phantom TokenType> has key, store { }

    /// A fixed time mint key which can mint token until global time > end_time
    struct FixedTimeMintKey<phantom TokenType> has key, store { total: u128, end_time: u64 }

    /// A linear time mint key which can mint token in a period by time-based linear release.
    struct LinearTimeMintKey<phantom TokenType> has key, store { total: u128, minted: u128, start_time: u64, period: u64 }

    /// A burn capability allows tokens of type `TokenType` to be burned.
    struct BurnCapability<phantom TokenType> has key, store { }


    /// Event emitted when token minted.
    struct MintEvent has drop, store {
        /// funds added to the system
        amount: u128,
        /// full info of Token.
        token_code: TokenCode,
    }

    /// Event emitted when token burned.
    struct BurnEvent has drop, store {
        /// funds removed from the system
        amount: u128,
        /// full info of Token
        token_code: TokenCode,
    }

    /// Token information.
    struct TokenInfo<phantom TokenType> has key {
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

    const EDEPRECATED_FUNCTION: u64 = 19;

    const EDESTROY_TOKEN_NON_ZERO: u64 = 16;
    const EINVALID_ARGUMENT: u64 = 18;

    /// Token register's address should same as TokenType's address.
    const ETOKEN_REGISTER: u64 = 101;

    const EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 102;
    // Mint key time limit
    const EMINT_KEY_TIME_LIMIT: u64 = 103;

    const EDESTROY_KEY_NOT_EMPTY: u64 = 104;
    const EPRECISION_TOO_LARGE: u64 = 105;
    const EEMPTY_KEY: u64 = 106;
    const ESPLIT: u64 = 107;
    const EPERIOD_NEW: u64 = 108;
    const EMINT_AMOUNT_EQUAL_ZERO: u64 = 109;

    /// 2^128 < 10**39
    const MAX_PRECISION: u8 = 38;

    /// Register the type `TokenType` as a Token and got MintCapability and BurnCapability.
    public fun register_token<TokenType: store>(
        account: &signer,
        precision: u8,
    ) {
        assert!(precision <= MAX_PRECISION, Errors::invalid_argument(EPRECISION_TOO_LARGE));
        let scaling_factor = Math::pow(10, (precision as u64));
        let token_address = token_address<TokenType>();
        assert!(Signer::address_of(account) == token_address, Errors::requires_address(ETOKEN_REGISTER));
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

    spec register_token {
        include RegisterTokenAbortsIf<TokenType>;
        include RegisterTokenEnsures<TokenType>;
    }

    spec schema RegisterTokenAbortsIf<TokenType> {
        precision: u8;
        account: signer;
        aborts_if precision > MAX_PRECISION;
        aborts_if Signer::address_of(account) != SPEC_TOKEN_TEST_ADDRESS();
        aborts_if exists<MintCapability<TokenType>>(Signer::address_of(account));
        aborts_if exists<BurnCapability<TokenType>>(Signer::address_of(account));
        aborts_if exists<TokenInfo<TokenType>>(Signer::address_of(account));
    }

    spec schema RegisterTokenEnsures<TokenType> {
        account: signer;
        ensures exists<MintCapability<TokenType>>(Signer::address_of(account));
        ensures exists<BurnCapability<TokenType>>(Signer::address_of(account));
        ensures exists<TokenInfo<TokenType>>(Signer::address_of(account));
    }

    /// Remove mint capability from `signer`.
    public fun remove_mint_capability<TokenType: store>(signer: &signer): MintCapability<TokenType>
    acquires MintCapability {
        move_from<MintCapability<TokenType>>(Signer::address_of(signer))
    }

    spec remove_mint_capability {
        aborts_if !exists<MintCapability<TokenType>>(Signer::address_of(signer));
        ensures !exists<MintCapability<TokenType>>(Signer::address_of(signer));
    }

    /// Add mint capability to `signer`.
    public fun add_mint_capability<TokenType: store>(signer: &signer, cap: MintCapability<TokenType>) {
        move_to(signer, cap)
    }

    spec add_mint_capability {
        aborts_if exists<MintCapability<TokenType>>(Signer::address_of(signer));
        ensures exists<MintCapability<TokenType>>(Signer::address_of(signer));
    }

    /// Destroy the given mint capability.
    public fun destroy_mint_capability<TokenType: store>(cap: MintCapability<TokenType>) {
        let MintCapability<TokenType> { } = cap;
    }

    spec destroy_mint_capability {
    }

    /// remove the token burn capability from `signer`.
    public fun remove_burn_capability<TokenType: store>(signer: &signer): BurnCapability<TokenType>
    acquires BurnCapability {
        move_from<BurnCapability<TokenType>>(Signer::address_of(signer))
    }

    spec remove_burn_capability {
        aborts_if !exists<BurnCapability<TokenType>>(Signer::address_of(signer));
        ensures !exists<BurnCapability<TokenType>>(Signer::address_of(signer));
    }

    /// Add token burn capability to `signer`.
    public fun add_burn_capability<TokenType: store>(signer: &signer, cap: BurnCapability<TokenType>) {
        move_to(signer, cap)
    }

    spec add_burn_capability {
        aborts_if exists<BurnCapability<TokenType>>(Signer::address_of(signer));
        ensures exists<BurnCapability<TokenType>>(Signer::address_of(signer));
    }

    /// Destroy the given burn capability.
    public fun destroy_burn_capability<TokenType: store>(cap: BurnCapability<TokenType>) {
        let BurnCapability<TokenType> { } = cap;
    }

    spec destroy_burn_capability {
    }

    /// Return `amount` tokens.
    /// Fails if the sender does not have a published MintCapability.
    public fun mint<TokenType: store>(account: &signer, amount: u128): Token<TokenType>
    acquires TokenInfo, MintCapability {
        mint_with_capability(
            borrow_global<MintCapability<TokenType>>(Signer::address_of(account)),
            amount,
        )
    }

    spec mint {
        aborts_if spec_abstract_total_value<TokenType>() + amount > MAX_U128;
        aborts_if !exists<MintCapability<TokenType>>(Signer::address_of(account));
    }

    /// Mint a new Token::Token worth `amount`.
    /// The caller must have a reference to a MintCapability.
    /// Only the Association account can acquire such a reference, and it can do so only via
    /// `borrow_sender_mint_capability`
    public fun mint_with_capability<TokenType: store>(
        _capability: &MintCapability<TokenType>,
        amount: u128,
    ): Token<TokenType> acquires TokenInfo {
        do_mint(amount)
    }

    spec mint_with_capability {
        aborts_if spec_abstract_total_value<TokenType>() + amount > MAX_U128;
        ensures spec_abstract_total_value<TokenType>() ==
                old(global<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS()).total_value) + amount;
    }

    fun do_mint<TokenType: store>(amount: u128): Token<TokenType> acquires TokenInfo {
        // update market cap resource to reflect minting
        let (token_address, module_name, token_name) = name_of_token<TokenType>();
        let info = borrow_global_mut<TokenInfo<TokenType>>(token_address);
        info.total_value = info.total_value + amount;
        Event::emit_event(
            &mut info.mint_events,
            MintEvent {
                amount,
                token_code: TokenCode { addr: token_address, module_name, name: token_name },
            },
        );
        Token<TokenType> { value: amount }
    }

    spec do_mint {
        aborts_if !exists<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS());
        aborts_if spec_abstract_total_value<TokenType>() + amount > MAX_U128;
    }

    /// Deprecated since @v3
    /// Issue a `FixedTimeMintKey` with given `MintCapability`.
    public fun issue_fixed_mint_key<TokenType: store>( _capability: &MintCapability<TokenType>,
                                     _amount: u128, _period: u64): FixedTimeMintKey<TokenType>{
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec issue_fixed_mint_key {
    }

    /// Deprecated since @v3
    /// Issue a `LinearTimeMintKey` with given `MintCapability`.
    public fun issue_linear_mint_key<TokenType: store>( _capability: &MintCapability<TokenType>,
                                                _amount: u128, _period: u64): LinearTimeMintKey<TokenType>{
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec issue_linear_mint_key {
    }

    /// Destroy `LinearTimeMintKey`, for deprecated
    public fun destroy_linear_time_key<TokenType: store>(key: LinearTimeMintKey<TokenType>): (u128, u128, u64, u64) {
        let LinearTimeMintKey<TokenType> { total, minted, start_time, period} = key;
        (total, minted, start_time, period)
    }

    public fun read_linear_time_key<TokenType: store>(key: &LinearTimeMintKey<TokenType>): (u128, u128, u64, u64) {
        (key.total, key.minted, key.start_time, key.period)
    }

    /// Burn some tokens of `signer`.
    public fun burn<TokenType: store>(account: &signer, tokens: Token<TokenType>)
    acquires TokenInfo, BurnCapability {
        burn_with_capability(
            borrow_global<BurnCapability<TokenType>>(Signer::address_of(account)),
            tokens,
        )
    }

    spec burn {
        aborts_if spec_abstract_total_value<TokenType>() - tokens.value < 0;
        aborts_if !exists<BurnCapability<TokenType>>(Signer::address_of(account));
    }

    /// Burn tokens with the given `BurnCapability`.
    public fun burn_with_capability<TokenType: store>(
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
                token_code: TokenCode { addr: token_address, module_name, name: token_name },
            },
        );
    }

    spec burn_with_capability {
        aborts_if spec_abstract_total_value<TokenType>() - tokens.value < 0;
        ensures spec_abstract_total_value<TokenType>() ==
                old(global<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS()).total_value) - tokens.value;
    }

    /// Create a new Token::Token<TokenType> with a value of 0
    public fun zero<TokenType: store>(): Token<TokenType> {
        Token<TokenType> { value: 0 }
    }

    spec zero {
    }


    /// Public accessor for the value of a token
    public fun value<TokenType: store>(token: &Token<TokenType>): u128 {
        token.value
    }

    spec value {
        aborts_if false;
    }

    /// Splits the given token into two and returns them both
    public fun split<TokenType: store>(
        token: Token<TokenType>,
        value: u128,
    ): (Token<TokenType>, Token<TokenType>) {
        let other = withdraw(&mut token, value);
        (token, other)
    }

    spec split {
        aborts_if token.value < value;
        ensures old(token.value) == result_1.value + result_2.value;
    }

    /// "Divides" the given token into two, where the original token is modified in place.
    /// The original token will have value = original value - `value`
    /// The new token will have a value = `value`
    /// Fails if the tokens value is less than `value`
    public fun withdraw<TokenType: store>(
        token: &mut Token<TokenType>,
        value: u128,
    ): Token<TokenType> {
        // Check that `value` is less than the token's value
        assert!(token.value >= value, Errors::limit_exceeded(EAMOUNT_EXCEEDS_COIN_VALUE));
        token.value = token.value - value;
        Token { value: value }
    }

    spec withdraw {
        aborts_if token.value < value;
        ensures result.value == value;
        ensures token.value == old(token).value - value;
    }

    /// Merges two tokens of the same token and returns a new token whose
    /// value is equal to the sum of the two inputs
    public fun join<TokenType: store>(
        token1: Token<TokenType>,
        token2: Token<TokenType>,
    ): Token<TokenType> {
        deposit(&mut token1, token2);
        token1
    }

    spec join {
        aborts_if token1.value + token2.value > max_u128();
        ensures old(token1).value + old(token2).value == result.value;
        ensures token1.value + token2.value == result.value;
    }

    /// "Merges" the two tokens
    /// The token passed in by reference will have a value equal to the sum of the two tokens
    /// The `check` token is consumed in the process
    public fun deposit<TokenType: store>(token: &mut Token<TokenType>, check: Token<TokenType>) {
        let Token { value } = check;
        token.value = token.value + value;
    }

    spec deposit {
        aborts_if token.value + check.value > max_u128();
        ensures old(token).value + check.value == token.value;
    }

    /// Destroy a token
    /// Fails if the value is non-zero
    /// The amount of Token in the system is a tightly controlled property,
    /// so you cannot "burn" any non-zero amount of Token
    public fun destroy_zero<TokenType: store>(token: Token<TokenType>) {
        let Token { value } = token;
        assert!(value == 0, Errors::invalid_state(EDESTROY_TOKEN_NON_ZERO))
    }

    spec destroy_zero {
        aborts_if token.value > 0;
    }

    /// Returns the scaling_factor for the `TokenType` token.
    public fun scaling_factor<TokenType: store>(): u128 acquires TokenInfo {
        let token_address = token_address<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).scaling_factor
    }

    spec scaling_factor {
        aborts_if false;
    }

    /// Return the total amount of token of type `TokenType`.
    public fun market_cap<TokenType: store>(): u128 acquires TokenInfo {
        let token_address = token_address<TokenType>();
        borrow_global<TokenInfo<TokenType>>(token_address).total_value
    }

    spec market_cap {
        aborts_if false;
    }

    /// Return true if the type `TokenType` is a registered in `token_address`.
    public fun is_registered_in<TokenType: store>(token_address: address): bool {
        exists<TokenInfo<TokenType>>(token_address)
    }

    spec is_registered_in {
        aborts_if false;
    }

    /// Return true if the type `TokenType1` is same with `TokenType2`
    public fun is_same_token<TokenType1: store, TokenType2: store>(): bool {
        return token_code<TokenType1>() == token_code<TokenType2>()
    }

    spec is_same_token {
        aborts_if false;
    }

    /// Return the TokenType's address
    public fun token_address<TokenType: store>(): address {
        let (addr, _, _) = name_of<TokenType>();
        addr
    }

    // The specification of this function is abstracted to avoid the complexity to
    // return a real address to caller
    spec token_address {
        pragma opaque = true;
        aborts_if false;
        ensures [abstract] exists<TokenInfo<TokenType>>(result);
        ensures [abstract] result == SPEC_TOKEN_TEST_ADDRESS();
        ensures [abstract] global<TokenInfo<TokenType>>(result).total_value == 100000000u128;
}

    /// Return the token code for the registered token.
    public fun token_code<TokenType: store>(): TokenCode {
        let (addr, module_name, name) = name_of<TokenType>();
        TokenCode {
            addr,
            module_name,
            name
        }
    }

    spec token_code {
        pragma opaque = true;
        aborts_if false;
        // ensures [abstract] result == spec_token_code<TokenType>();
    }

    /// We use an uninterpreted function to represent the result of derived address. The actual value
    /// does not matter for the verification of callers.
    spec fun spec_token_code<TokenType>(): TokenCode;

    /// Return Token's module address, module name, and type name of `TokenType`.
    native fun name_of<TokenType: store>(): (address, vector<u8>, vector<u8>);

    spec name_of {
        pragma opaque = true;
        aborts_if false;
    }

    fun name_of_token<TokenType: store>(): (address, vector<u8>, vector<u8>) {
        name_of<TokenType>()
    }

    // The specification of this function is abstracted to avoid the complexity to
    // return a real address to caller
    spec name_of_token {
        pragma opaque = true;
        aborts_if false;
        ensures [abstract] exists<TokenInfo<TokenType>>(result_1);
        ensures [abstract] result_1 == SPEC_TOKEN_TEST_ADDRESS();
        ensures [abstract] global<TokenInfo<TokenType>>(result_1).total_value == 100000000u128;
    }


    spec fun SPEC_TOKEN_TEST_ADDRESS(): address {
        @0x2
    }

    spec fun spec_abstract_total_value<TokenType>(): u128 {
        global<TokenInfo<TokenType>>(SPEC_TOKEN_TEST_ADDRESS()).total_value
    }


}
}