address 0x1 {

module Coin {
    use 0x1::Event;
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::RegisteredCurrencies;
    use 0x1::Vector;
    use 0x1::Generic;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    // The currency has a `CoinType` color that tells us what currency the
    // `value` inside represents.
    resource struct Coin<CoinType> { value: u64 }

    // A minting capability allows coins of type `CoinType` to be minted
    resource struct MintCapability<CoinType> { }

    // A burn capability allows coins of type `CoinType` to be burned
    resource struct BurnCapability<CoinType> { }

    struct MintEvent {
        // funds added to the system
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "STC")
        currency_code: vector<u8>,
    }

    struct BurnEvent {
        // funds removed from the system
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "STC")
        currency_code: vector<u8>,
        // address with the Preburn resource that stored the now-burned funds
        preburn_address: address,
    }

    struct PreburnEvent {
        // funds waiting to be removed from the system
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "STC")
        currency_code: vector<u8>,
        // address with the Preburn resource that now holds the funds
        preburn_address: address,
    }

    struct CancelBurnEvent {
        // funds returned
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "STC")
        currency_code: vector<u8>,
        // address with the Preburn resource that holds the now-returned funds
        preburn_address: address,
    }

    // The information for every supported currency is stored in a resource
    // under the `issuer_addr()` address. Unless they are specified
    // otherwise the fields in this resource are immutable.
    resource struct CurrencyInfo<CoinType> {
        // The total value for the currency represented by
        // `CoinType`. Mutable.
        total_value: u128,
        // Value of funds that are in the process of being burned
        preburn_value: u64,
        // The (rough) exchange rate from `CoinType` to STC.
        // For support pay custom Token as gas.
        to_stc_exchange_rate: FixedPoint32,
        //TODO remove this.
        is_synthetic: bool,
        // The scaling factor for the coin (i.e. the amount to multiply by
        // to get to the human-readable reprentation for this currency). e.g. 10^6 for Coin1
        scaling_factor: u64,
        // The smallest fractional part (number of decimal places) to be
        // used in the human-readable representation for the currency (e.g.
        // 10^2 for Coin1 cents)
        fractional_part: u64,
        // The code symbol for this `CoinType`. UTF-8 encoded.
        // e.g. for "STC" this is x"4C4252". No character limit.
        currency_code: vector<u8>,
        // We may want to disable the ability to mint further coins of a
        // currency while that currency is still around. Mutable.
        can_mint: bool,
        // event stream for minting
        mint_events: Event::EventHandle<MintEvent>,
        // event stream for burning
        burn_events: Event::EventHandle<BurnEvent>,
        // event stream for preburn requests
        preburn_events: Event::EventHandle<PreburnEvent>,
        // event stream for cancelled preburn requests
        cancel_burn_events: Event::EventHandle<CancelBurnEvent>,
    }

    // A holding area where funds that will subsequently be burned wait while their underyling
    // assets are sold off-chain.
    // This resource can only be created by the holder of the BurnCapability. An account that
    // contains this address has the authority to initiate a burn request. A burn request can be
    // resolved by the holder of the BurnCapability by either (1) burning the funds, or (2)
    // returning the funds to the account that initiated the burn request.
    // This design supports multiple preburn requests in flight at the same time, including multiple
    // burn requests from the same account. However, burn requests from the same account must be
    // resolved in FIFO order.
    resource struct Preburn<Token> {
        // Queue of pending burn requests
        requests: vector<Coin<Token>>,
        // Boolean that is true if the holder of the BurnCapability has approved this account as a
        // preburner
        is_approved: bool,
    }

    // An association account holding this privilege can add/remove the
    // currencies from the system.
    struct AddCurrency { }

    ///////////////////////////////////////////////////////////////////////////
    // Initialization and granting of privileges
    ///////////////////////////////////////////////////////////////////////////

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 0);
        RegisteredCurrencies::initialize(account);
    }

    // Returns a MintCapability for the `CoinType` currency. `CoinType`
    // must be a registered currency type.
    public fun grant_mint_capability<CoinType>(account: &signer): MintCapability<CoinType> {
        assert_issuer_and_currency<CoinType>(account);
        MintCapability<CoinType> { }
    }

    // Returns a `BurnCapability` for the `CoinType` currency. `CoinType`
    // must be a registered currency type.
    public fun grant_burn_capability<CoinType>(account: &signer): BurnCapability<CoinType> {
        assert_issuer_and_currency<CoinType>(account);
        BurnCapability<CoinType> { }
    }

    public fun grant_burn_capability_for_sender<CoinType>(account: &signer) {
        //assert(Signer::address_of(account) == 0xD1E, 0);
        move_to(account,grant_burn_capability<CoinType>(account));
    }

    // Return `amount` coins.
    // Fails if the sender does not have a published MintCapability.
    public fun mint<Token>(account: &signer, amount: u64): Coin<Token> acquires CurrencyInfo, MintCapability {
        mint_with_capability(amount, borrow_global<MintCapability<Token>>(Signer::address_of(account)))
    }

    // Burn the coins currently held in the preburn holding area under `preburn_address`.
    // Fails if the sender does not have a published `BurnCapability`.
    public fun burn<Token>(account: &signer,
        preburn_address: address
    ) acquires BurnCapability, CurrencyInfo, Preburn {
        burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<Token>>(Signer::address_of(account))
        )
    }

    // Cancel the oldest burn request from `preburn_address`
    // Fails if the sender does not have a published `BurnCapability`.
    public fun cancel_burn<Token>(
        account: &signer,
        preburn_address: address
    ): Coin<Token> acquires BurnCapability, CurrencyInfo, Preburn {
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<Token>>(Signer::address_of(account))
        )
    }

    public fun new_preburn<Token>(): Preburn<Token> {
        assert_is_coin<Token>();
        Preburn<Token> { requests: Vector::empty(), is_approved: false, }
    }

    // Mint a new Coin::Coin worth `value`. The caller must have a reference to a MintCapability.
    // Only the Association account can acquire such a reference, and it can do so only via
    // `borrow_sender_mint_capability`
    public fun mint_with_capability<Token>(
        value: u64,
        _capability: &MintCapability<Token>
    ): Coin<Token> acquires CurrencyInfo {
        assert_is_coin<Token>();
        // TODO: temporary measure for testnet only: limit minting to 1B Libra at a time.
        // this is to prevent the market cap's total value from hitting u64_max due to excessive
        // minting. This will not be a problem in the production Libra system because coins will
        // be backed with real-world assets, and thus minting will be correspondingly rarer.
        // * 1000000 here because the unit is microlibra
        // assert(value <= 1000000000 * 1000000, 11);
        let currency_code = currency_code<Token>();
        // update market cap resource to reflect minting
        let info = borrow_global_mut<CurrencyInfo<Token>>(issuer_addr<Token>());
        assert(info.can_mint, 4);
        info.total_value = info.total_value + (value as u128);
        // don't emit mint events for synthetic currenices
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.mint_events,
                MintEvent{
                    amount: value,
                    currency_code,
                }
            );
        };

        Coin<Token> { value }
    }

    // Create a new Preburn resource.
    // Can only be called by the holder of the BurnCapability.
    public fun new_preburn_with_capability<Token>(
        _capability: &BurnCapability<Token>
    ): Preburn<Token> {
        assert_is_coin<Token>();
        Preburn<Token> { requests: Vector::empty(), is_approved: true }
    }

    // Send a coin to the preburn holding area `preburn` that is passed in.
    public fun preburn_with_resource<Token>(
        coin: Coin<Token>,
        preburn: &mut Preburn<Token>,
        preburn_address: address,
    ) acquires CurrencyInfo {
        let coin_value = value(&coin);
        Vector::push_back(
            &mut preburn.requests,
            coin
        );
        let currency_code = currency_code<Token>();
        let info = borrow_global_mut<CurrencyInfo<Token>>(issuer_addr<Token>());
        info.preburn_value = info.preburn_value + coin_value;
        // don't emit preburn events for synthetic currencies
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.preburn_events,
                PreburnEvent{
                    amount: coin_value,
                    currency_code,
                    preburn_address,
                }
            );
        };
    }

    // Send coin to the preburn holding area, where it will wait to be burned.
    // Fails if the sender does not have a published Preburn resource
    public fun preburn_to_sender<Token>(account: &signer, coin: Coin<Token>) acquires CurrencyInfo, Preburn {
        let sender = Signer::address_of(account);
        preburn_with_resource(coin, borrow_global_mut<Preburn<Token>>(sender), sender);
    }

    // Permanently remove the coins held in the `Preburn` resource stored at `preburn_address` and
    // update the market cap accordingly. If there are multiple preburn requests in progress, this
    // will remove the oldest one.
    // Can only be invoked by the holder of the `BurnCapability`. Fails if the there is no `Preburn`
    // resource under `preburn_address` or has one with no pending burn requests.
    public fun burn_with_capability<Token>(
        preburn_address: address,
        capability: &BurnCapability<Token>
    ) acquires CurrencyInfo, Preburn {
        // destroy the coin at the head of the preburn queue
        burn_with_resource_cap(
            borrow_global_mut<Preburn<Token>>(preburn_address),
            preburn_address,
            capability
        )
    }

    // Permanently remove the coins held in the passed-in preburn resource
    // and update the market cap accordingly. If there are multiple preburn
    // requests in progress, this will remove the oldest one.
    // Can only be invoked by the holder of the `BurnCapability`. Fails if
    // the `preburn` resource has no pending burn requests.
    public fun burn_with_resource_cap<Token>(
        preburn: &mut Preburn<Token>,
        preburn_address: address,
        _capability: &BurnCapability<Token>
    ) acquires CurrencyInfo {
        // destroy the coin at the head of the preburn queue
        let Coin { value } = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let currency_code = currency_code<Token>();
        let info = borrow_global_mut<CurrencyInfo<Token>>(issuer_addr<Token>());
        info.total_value = info.total_value - (value as u128);
        info.preburn_value = info.preburn_value - value;
        // don't emit burn events for synthetic currencies
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.burn_events,
                BurnEvent {
                    amount: value,
                    currency_code,
                    preburn_address,
                }
            );
        };
    }

    // Cancel the burn request in the `Preburn` resource stored at `preburn_address` and
    // return the coins to the caller.
    // If there are multiple preburn requests in progress, this will cancel the oldest one.
    // Can only be invoked by the holder of the `BurnCapability`. Fails if the transaction sender
    // does not have a published Preburn resource or has one with no pending burn requests.
    public fun cancel_burn_with_capability<Token>(
        preburn_address: address,
        _capability: &BurnCapability<Token>
    ): Coin<Token> acquires CurrencyInfo, Preburn {
        // destroy the coin at the head of the preburn queue
        let preburn = borrow_global_mut<Preburn<Token>>(preburn_address);
        let coin = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let currency_code = currency_code<Token>();
        let info = borrow_global_mut<CurrencyInfo<Token>>(issuer_addr<Token>());
        let amount = value(&coin);
        info.preburn_value = info.preburn_value - amount;
        // Don't emit cancel burn events for synthetic currencies. cancel burn shouldn't be be used
        // for synthetics in the first place
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.cancel_burn_events,
                CancelBurnEvent {
                    amount,
                    currency_code,
                    preburn_address,
                }
            );
        };

        coin
    }

    // Publish `preburn` under the sender's account
    public fun publish_preburn<Token>(account: &signer, preburn: Preburn<Token>) {
        move_to(account,preburn)
    }

    // Publish `capability` under the sender's account
    public fun publish_mint_capability<Token>(account: &signer, capability: MintCapability<Token>) {
        move_to(account,capability)
    }

    // Remove and return the `Preburn` resource under the sender's account
    public fun remove_preburn<Token>(account: &signer): Preburn<Token> acquires Preburn {
        move_from<Preburn<Token>>(Signer::address_of(account))
    }

    // Destroys the given preburn resource.
    // Aborts if `requests` is non-empty
    public fun destroy_preburn<Token>(preburn: Preburn<Token>) {
        let Preburn { requests, is_approved: _ } = preburn;
        Vector::destroy_empty(requests)
    }

    // Remove and return the MintCapability from the sender's account. Fails if the sender does
    // not have a published MintCapability
    public fun remove_mint_capability<Token>(account: &signer): MintCapability<Token> acquires MintCapability {
        move_from<MintCapability<Token>>(Signer::address_of(account))
    }

    // Remove and return the BurnCapability from the sender's account. Fails if the sender does
    // not have a published BurnCapability
    public fun remove_burn_capability<Token>(account: &signer): BurnCapability<Token> acquires BurnCapability {
        move_from<BurnCapability<Token>>(Signer::address_of(account))
    }

    // Return the total value of Libra to be burned
    public fun preburn_value<Token>(): u64 acquires CurrencyInfo {
        borrow_global<CurrencyInfo<Token>>(issuer_addr<Token>()).preburn_value
    }

    // Create a new Coin::Coin<CoinType> with a value of 0
    public fun zero<CoinType>(): Coin<CoinType> {
        assert_is_coin<CoinType>();
        Coin<CoinType> { value: 0 }
    }

    // Public accessor for the value of a coin
    public fun value<CoinType>(coin: &Coin<CoinType>): u64 {
        coin.value
    }

    // Splits the given coin into two and returns them both
    // It leverages `Self::withdraw` for any verifications of the values
    public fun split<CoinType>(coin: Coin<CoinType>, amount: u64): (Coin<CoinType>, Coin<CoinType>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    // "Divides" the given coin into two, where the original coin is modified in place
    // The original coin will have value = original value - `amount`
    // The new coin will have a value = `amount`
    // Fails if the coins value is less than `amount`
    public fun withdraw<CoinType>(coin: &mut Coin<CoinType>, amount: u64): Coin<CoinType> {
        // Check that `amount` is less than the coin's value
        assert(coin.value >= amount, 10);
        coin.value = coin.value - amount;
        Coin { value: amount }
    }

    /// Return a `Coin<CoinType>` worth `coin.value` and reduces the `value` of the input `coin` to
    /// zero. Does not abort.
    public fun withdraw_all<CoinType>(coin: &mut Coin<CoinType>): Coin<CoinType> {
        let val = coin.value;
        withdraw(coin, val)
    }

    // Merges two coins of the same currency and returns a new coin whose
    // value is equal to the sum of the two inputs
    public fun join<CoinType>(coin1: Coin<CoinType>, coin2: Coin<CoinType>): Coin<CoinType>  {
        deposit(&mut coin1, coin2);
        coin1
    }

    // "Merges" the two coins
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit<CoinType>(coin: &mut Coin<CoinType>, check: Coin<CoinType>) {
        let Coin { value } = check;
        coin.value = coin.value + value;
    }

    // Destroy a coin
    // Fails if the value is non-zero
    // The amount of Coin in the system is a tightly controlled property,
    // so you cannot "burn" any non-zero amount of Coin
    public fun destroy_zero<CoinType>(coin: Coin<CoinType>) {
        let Coin { value } = coin;
        assert(value == 0, 5)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Definition of Currencies
    ///////////////////////////////////////////////////////////////////////////

    // Register the type `CoinType` as a currency. Without this, a type
    // cannot be used as a coin/currency unit n Libra.
    public fun register_currency<CoinType>(account: &signer,
        to_stc_exchange_rate: FixedPoint32,
        scaling_factor: u64,
        fractional_part: u64,
    ) {
        // And only callable by the designated currency address.
        //assert(Association::has_privilege<AddCurrency>(Signer::address_of(account)), 8);
        assert_issuer<CoinType>(account);
        let (_coin_module_address,coin_module_name,struct_name) = Generic::type_of<CoinType>();
        // CoinType's struct name must be same as Coin Name. TODO consider a more graceful approach.
        assert(struct_name == copy coin_module_name, 8);
        move_to(account,MintCapability<CoinType>{});
        move_to(account,BurnCapability<CoinType>{});
        move_to(account,CurrencyInfo<CoinType> {
            total_value: 0,
            preburn_value: 0,
            to_stc_exchange_rate,
            is_synthetic: false,
            scaling_factor,
            fractional_part,
            currency_code: copy coin_module_name,
            can_mint: true,
            mint_events: Event::new_event_handle<MintEvent>(account),
            burn_events: Event::new_event_handle<BurnEvent>(account),
            preburn_events: Event::new_event_handle<PreburnEvent>(account),
            cancel_burn_events: Event::new_event_handle<CancelBurnEvent>(account)
        });
        RegisteredCurrencies::add_currency_code(
            account,
            coin_module_name
        )
    }

    // Return the total amount of currency minted of type `CoinType`
    public fun market_cap<CoinType>(): u128
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(issuer_addr<CoinType>()).total_value
    }

    // Returns the value of the coin in the `FromCoinType` currency in STC.
    // This should only be used where a _rough_ approximation of the exchange
    // rate is needed.
    public fun approx_stc_for_value<FromCoinType>(from_value: u64): u64
    acquires CurrencyInfo {
        let stc_exchange_rate = stc_exchange_rate<FromCoinType>();
        FixedPoint32::multiply_u64(from_value, stc_exchange_rate)
    }

    // Returns the value of the coin in the `FromCoinType` currency in STC.
    // This should only be used where a rough approximation of the exchange
    // rate is needed.
    public fun approx_stc_for_coin<FromCoinType>(coin: &Coin<FromCoinType>): u64
    acquires CurrencyInfo {
        let from_value = value(coin);
        approx_stc_for_value<FromCoinType>(from_value)
    }

    // Return true if the type `CoinType` is a registered currency.
    public fun is_currency<CoinType>(): bool {
        exists<CurrencyInfo<CoinType>>(issuer_addr<CoinType>())
    }

    // Predicate on whether `CoinType` is a synthetic currency.
    public fun is_synthetic_currency<CoinType>(): bool
    acquires CurrencyInfo {
        let addr = issuer_addr<CoinType>();
        exists<CurrencyInfo<CoinType>>(addr) &&
            borrow_global<CurrencyInfo<CoinType>>(addr).is_synthetic
    }

    // Returns the scaling factor for the `CoinType` currency.
    public fun scaling_factor<CoinType>(): u64
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(issuer_addr<CoinType>()).scaling_factor
    }

    // Returns the representable fractional part for the `CoinType` currency.
    public fun fractional_part<CoinType>(): u64
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(issuer_addr<CoinType>()).fractional_part
    }

    // Return the currency code for the registered currency.
    public fun currency_code<CoinType>(): vector<u8>
    acquires CurrencyInfo {
        *&borrow_global<CurrencyInfo<CoinType>>(issuer_addr<CoinType>()).currency_code
    }

    // Updates the exchange rate for `FromCoinType` to STC exchange rate held on chain.
    public fun update_stc_exchange_rate<FromCoinType>(account: &signer, stc_exchange_rate: FixedPoint32)
    acquires CurrencyInfo {
        assert_issuer_and_currency<FromCoinType>(account);
        let currency_info = borrow_global_mut<CurrencyInfo<FromCoinType>>(issuer_addr<FromCoinType>());
        currency_info.to_stc_exchange_rate = stc_exchange_rate;
    }

    // Return the (rough) exchange rate between `CoinType` and STC
    public fun stc_exchange_rate<CoinType>(): FixedPoint32
    acquires CurrencyInfo {
        *&borrow_global<CurrencyInfo<CoinType>>(issuer_addr<CoinType>()).to_stc_exchange_rate
    }

    // There may be situations in which we disallow the further minting of
    // coins in the system without removing the currency. This function
    // allows the association to control whether or not further coins of
    // `CoinType` can be minted or not.
    public fun update_minting_ability<CoinType>(account: &signer, can_mint: bool)
    acquires CurrencyInfo {
        assert_issuer_and_currency<CoinType>(account);
        let currency_info = borrow_global_mut<CurrencyInfo<CoinType>>(Signer::address_of(account));
        currency_info.can_mint = can_mint;
    }


    ///////////////////////////////////////////////////////////////////////////
    // Helper functions
    ///////////////////////////////////////////////////////////////////////////

    // The (singleton) address under which the currency registration
    // information is published.
    fun issuer_addr<CoinType>(): address {
        let (coin_type_addr, _,_) = Generic::type_of<CoinType>();
        coin_type_addr
    }

    fun assert_issuer<CoinType>(account: &signer){
        let issuer_addr = issuer_addr<CoinType>();
        assert(issuer_addr == Signer::address_of(account), 8);
    }

    public fun assert_issuer_and_currency<CoinType>(account: &signer){
        assert_issuer<CoinType>(account);
        assert_is_coin<CoinType>();
    }

    // Assert that `CoinType` is a registered currency
    fun assert_is_coin<CoinType>() {
        assert(is_currency<CoinType>(), 1);
    }
}

}
