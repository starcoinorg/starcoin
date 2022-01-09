address StarcoinFramework {
/// The module for the Treasury of DAO, which can hold the token of DAO.
module Treasury {
     use StarcoinFramework::Event;
     use StarcoinFramework::Signer;
     use StarcoinFramework::Errors;
     use StarcoinFramework::Timestamp;
     use StarcoinFramework::Math;
     use StarcoinFramework::Token::{Self,Token};

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
    }

    struct Treasury<phantom TokenT> has store,key {
        balance: Token<TokenT>,
        /// event handle for treasury withdraw event
        withdraw_events: Event::EventHandle<WithdrawEvent>,
        /// event handle for treasury deposit event
        deposit_events: Event::EventHandle<DepositEvent>,
    }
    
    /// A withdraw capability allows tokens of type `TokenT` to be withdraw from Treasury.
    struct WithdrawCapability<phantom TokenT> has key, store { }
    
    /// A linear time withdraw capability which can withdraw token from Treasury in a period by time-based linear release.
    struct LinearWithdrawCapability<phantom TokenT> has key, store {
        /// The total amount of tokens that can be withdrawn by this capability
        total: u128,
        /// The amount of tokens that have been withdrawn by this capability
        withdraw: u128,
        /// The time-based linear release start time, timestamp in seconds.
        start_time: u64,
        ///  The time-based linear release period in seconds
        period: u64
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
    

    /// Init a Treasury for TokenT,can only be called by token issuer.
    public fun initialize<TokenT:store>(signer: &signer, init_token: Token<TokenT>) :WithdrawCapability<TokenT> {
        let token_issuer = Token::token_address<TokenT>();
        assert!(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let treasure = Treasury{
            balance: init_token,
            withdraw_events: Event::new_event_handle<WithdrawEvent>(signer),
            deposit_events: Event::new_event_handle<DepositEvent>(signer),
        };
        move_to(signer,treasure);
        WithdrawCapability<TokenT>{}
    }

    spec initialize {
    }

    /// Check the Treasury of TokenT is exists.
    public fun exists_at<TokenT:store>(): bool {
        let token_issuer = Token::token_address<TokenT>();
        exists<Treasury<TokenT>>(token_issuer)
    }

    spec exists_at {
    }

    /// Get the balance of TokenT's Treasury
    /// if the Treasury do not exists, return 0.
    public fun balance<TokenT:store>(): u128 acquires Treasury{
        let token_issuer = Token::token_address<TokenT>();
        if(!exists<Treasury<TokenT>>(token_issuer)){
            return 0
        };
        let treasury = borrow_global<Treasury<TokenT>>(token_issuer);
        Token::value(&treasury.balance)
    }

    spec balance {
    }

    public fun deposit<TokenT:store>(token: Token<TokenT>) acquires Treasury{
        assert!(exists_at<TokenT>(), Errors::not_published(ERR_TREASURY_NOT_EXIST));
        let token_address = Token::token_address<TokenT>();
        let treasury = borrow_global_mut<Treasury<TokenT>>(token_address);
        let amount = Token::value(&token);
        Event::emit_event(
                &mut treasury.deposit_events,
                DepositEvent {
                    amount,
                },
            );
        Token::deposit(&mut treasury.balance, token);
    }

    spec deposit {
    }

    fun do_withdraw<TokenT:store>(amount: u128): Token<TokenT> acquires Treasury {
        assert!(amount > 0, Errors::invalid_argument(ERR_ZERO_AMOUNT));
        assert!(exists_at<TokenT>(), Errors::not_published(ERR_TREASURY_NOT_EXIST));
        let token_address = Token::token_address<TokenT>();
        let treasury = borrow_global_mut<Treasury<TokenT>>(token_address);
        assert!(amount <= Token::value(&treasury.balance) , Errors::invalid_argument(ERR_TOO_BIG_AMOUNT));
        Event::emit_event(
            &mut treasury.withdraw_events,
            WithdrawEvent {
                amount,
            },
        );
        Token::withdraw(&mut treasury.balance, amount)
    }
    
    spec do_withdraw {
        aborts_if !exists<Treasury<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }

    /// Withdraw tokens with given `LinearWithdrawCapability`.
    public fun withdraw_with_capability<TokenT:store>(_cap: &mut WithdrawCapability<TokenT>, amount: u128): Token<TokenT> acquires Treasury {
        let token = do_withdraw(amount);
        token
    }

    spec withdraw_with_capability {
    }

    /// Withdraw from TokenT's  treasury, the signer must have WithdrawCapability<TokenT>
    public fun withdraw<TokenT:store>(signer: &signer, amount: u128) : Token<TokenT> acquires Treasury, WithdrawCapability{
        let cap = borrow_global_mut<WithdrawCapability<TokenT>>(Signer::address_of(signer));
        Self::withdraw_with_capability(cap, amount)
    }

    spec withdraw {
    }
  
    /// Issue a `LinearWithdrawCapability` with given `WithdrawCapability`.
    public fun issue_linear_withdraw_capability<TokenT: store>( _capability: &mut WithdrawCapability<TokenT>,
                                                amount: u128, period: u64): LinearWithdrawCapability<TokenT>{
        assert!(period > 0, Errors::invalid_argument(ERR_INVALID_PERIOD));
        assert!(amount > 0, Errors::invalid_argument(ERR_ZERO_AMOUNT));
        let start_time = Timestamp::now_seconds();
        LinearWithdrawCapability<TokenT> {
            total: amount,
            withdraw: 0,
            start_time,
            period
        }
    }
    
    spec issue_linear_withdraw_capability {
        aborts_if period == 0;
        aborts_if amount == 0;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(StarcoinFramework::CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
    
    /// Withdraw tokens with given `LinearWithdrawCapability`.
    public fun withdraw_with_linear_capability<TokenT: store>(cap: &mut LinearWithdrawCapability<TokenT>): Token<TokenT> acquires Treasury {
        let amount = withdraw_amount_of_linear_cap(cap);
        let token = do_withdraw(amount);
        cap.withdraw = cap.withdraw + amount;
        token
    }

    spec withdraw_with_linear_capability {
        pragma verify = false; //timeout, fix later
    }

    /// Withdraw from TokenT's  treasury, the signer must have LinearWithdrawCapability<TokenT>
    public fun withdraw_by_linear<TokenT:store>(signer: &signer) : Token<TokenT> acquires Treasury, LinearWithdrawCapability{
        let cap = borrow_global_mut<LinearWithdrawCapability<TokenT>>(Signer::address_of(signer));
        Self::withdraw_with_linear_capability(cap)
    }
    
    /// Split the given `LinearWithdrawCapability`.
    public fun split_linear_withdraw_cap<TokenT: store>(cap: &mut LinearWithdrawCapability<TokenT>, amount: u128): (Token<TokenT>, LinearWithdrawCapability<TokenT>) acquires Treasury {
        assert!(amount > 0, Errors::invalid_argument(ERR_ZERO_AMOUNT));
        let token = Self::withdraw_with_linear_capability(cap);
        assert!((cap.withdraw + amount) <= cap.total, Errors::invalid_argument(ERR_TOO_BIG_AMOUNT));
        cap.total = cap.total - amount;
        let start_time = Timestamp::now_seconds();
        let new_period = cap.start_time + cap.period - start_time;
        let new_key = LinearWithdrawCapability<TokenT> {
            total: amount,
            withdraw: 0,
            start_time,
            period: new_period
        };
        (token, new_key)
    }

    spec split_linear_withdraw_cap {
        pragma verify = false; //timeout, fix later
    }
        
        
    /// Returns the amount of the LinearWithdrawCapability can mint now.
    public fun withdraw_amount_of_linear_cap<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>): u128 {
        let now = Timestamp::now_seconds();
        let elapsed_time = now - cap.start_time;
        if (elapsed_time >= cap.period) {
            cap.total - cap.withdraw
        }else {
            Math::mul_div(cap.total, (elapsed_time as u128), (cap.period as u128)) - cap.withdraw
        }
    }
        
    spec withdraw_amount_of_linear_cap {
        pragma verify = false; //timeout, fix later
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(StarcoinFramework::CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if Timestamp::spec_now_seconds() < cap.start_time;
        aborts_if Timestamp::spec_now_seconds() - cap.start_time >= cap.period && cap.total < cap.withdraw;
        aborts_if [abstract] Timestamp::spec_now_seconds() - cap.start_time < cap.period && Math::spec_mul_div() < cap.withdraw;
    }
    
    /// Check if the given `LinearWithdrawCapability` is empty.
    public fun is_empty_linear_withdraw_cap<TokenT:store>(key: &LinearWithdrawCapability<TokenT>) : bool {
        key.total == key.withdraw
    }

    spec is_empty_linear_withdraw_cap {
        aborts_if false;
    }

     /// Remove mint capability from `signer`.
    public fun remove_withdraw_capability<TokenT: store>(signer: &signer): WithdrawCapability<TokenT>
    acquires WithdrawCapability {
        move_from<WithdrawCapability<TokenT>>(Signer::address_of(signer))
    }

    spec remove_withdraw_capability {
        aborts_if !exists<WithdrawCapability<TokenT>>(Signer::address_of(signer));
        ensures !exists<WithdrawCapability<TokenT>>(Signer::address_of(signer));
    }

    /// Save mint capability to `signer`.
    public fun add_withdraw_capability<TokenT: store>(signer: &signer, cap: WithdrawCapability<TokenT>) {
        move_to(signer, cap)
    }

    spec add_withdraw_capability {
        aborts_if exists<WithdrawCapability<TokenT>>(Signer::address_of(signer));
        ensures exists<WithdrawCapability<TokenT>>(Signer::address_of(signer));
    }

    /// Destroy the given mint capability.
    public fun destroy_withdraw_capability<TokenT: store>(cap: WithdrawCapability<TokenT>) {
        let WithdrawCapability<TokenT> { } = cap;
    }

    spec destroy_withdraw_capability {
    }

    /// Add LinearWithdrawCapability to `signer`, a address only can have one LinearWithdrawCapability<T>
    public fun add_linear_withdraw_capability<TokenT: store>(signer: &signer, cap: LinearWithdrawCapability<TokenT>){
        move_to(signer, cap)
    }

    /// Remove LinearWithdrawCapability from `signer`.
    public fun remove_linear_withdraw_capability<TokenT: store>(signer: &signer): LinearWithdrawCapability<TokenT>
    acquires LinearWithdrawCapability {
        move_from<LinearWithdrawCapability<TokenT>>(Signer::address_of(signer))
    }

    /// Destroy LinearWithdrawCapability.
    public fun destroy_linear_withdraw_capability<TokenT: store>(cap: LinearWithdrawCapability<TokenT>) {
        let LinearWithdrawCapability{ total: _, withdraw: _, start_time: _, period: _ } = cap;
    }

    public fun is_empty_linear_withdraw_capability<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>):bool {
        cap.total == cap.withdraw
    }

    /// Get LinearWithdrawCapability total amount
    public fun get_linear_withdraw_capability_total<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>):u128 {
        cap.total
    }

    /// Get LinearWithdrawCapability withdraw amount
    public fun get_linear_withdraw_capability_withdraw<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>):u128 {
        cap.withdraw
    }

    /// Get LinearWithdrawCapability period in seconds
    public fun get_linear_withdraw_capability_period<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>):u64 {
        cap.period
    }

    /// Get LinearWithdrawCapability start_time in seconds
    public fun get_linear_withdraw_capability_start_time<TokenT: store>(cap: &LinearWithdrawCapability<TokenT>):u64 {
        cap.start_time
    }


    spec fun spec_balance<TokenType>(): u128 {
        global<Treasury<TokenType>>(Token::SPEC_TOKEN_TEST_ADDRESS()).balance.value
    }

}
}