address 0x1 {

// The module for the account resource that governs every account
module Account {
    use 0x1::Authenticator;
    use 0x1::Event;
    use 0x1::Hash;
    use 0x1::Token::{Self, Token};
    use 0x1::Vector;
    use 0x1::Signer;
    use 0x1::Timestamp;
    use 0x1::Option::{Self, Option};
    use 0x1::TransactionFee;
    use 0x1::CoreAddresses;
    use 0x1::ErrorCode;
    use 0x1::STC::{Self, STC};

    spec module {
        pragma verify;
        pragma aborts_if_is_strict = true;
    }

    // Every account has a Account::Account resource
    resource struct Account {
        // The current authentication key.
        // This can be different than the key used to create the account
        authentication_key: vector<u8>,
        // A `withdrawal_capability` allows whoever holds this capability
        // to withdraw from the account. At the time of account creation
        // this capability is stored in this option. It can later be
        // "extracted" from this field via `extract_withdraw_capability`,
        // and can also be restored via `restore_withdraw_capability`.
        withdrawal_capability: Option<WithdrawCapability>,
        // A `key_rotation_capability` allows whoever holds this capability
        // the ability to rotate the authentication key for the account. At
        // the time of account creation this capability is stored in this
        // option. It can later be "extracted" from this field via
        // `extract_key_rotation_capability`, and can also be restored via
        // `restore_key_rotation_capability`.
        key_rotation_capability: Option<KeyRotationCapability>,
        // Event handle for received event
        received_events: Event::EventHandle<ReceivedPaymentEvent>,
        // Event handle for sent event
        sent_events: Event::EventHandle<SentPaymentEvent>,
        // Event handle for accept_token event
        accept_token_events: Event::EventHandle<AcceptTokenEvent>,
        // The current sequence number.
        // Incremented by one each time a transaction is submitted
        sequence_number: u64,
    }

    // A resource that holds the tokens stored in this account
    resource struct Balance<TokenType> {
        token: Token<TokenType>,
    }

    // The holder of WithdrawCapability for account_address can withdraw Token from
    // account_address/Account::Account/balance.
    // There is at most one WithdrawCapability in existence for a given address.
    resource struct WithdrawCapability {
        account_address: address,
    }

    // The holder of KeyRotationCapability for account_address can rotate the authentication key for
    // account_address (i.e., write to account_address/Account::Account/authentication_key).
    // There is at most one KeyRotationCapability in existence for a given address.
    resource struct KeyRotationCapability {
        account_address: address,
    }

    // Message for sent events
    struct SentPaymentEvent {
        // The amount of Token<TokenType> sent
        amount: u128,
        // The code symbol for the token that was sent
        token_code: vector<u8>,
        // The address that was paid
        payee: address,
        // Metadata associated with the payment
        metadata: vector<u8>,
    }

    // Message for received events
    struct ReceivedPaymentEvent {
        // The amount of Token<TokenType> received
        amount: u128,
        // The code symbol for the token that was received
        token_code: vector<u8>,
        // The address that sent the token
        payer: address,
        // Metadata associated with the payment
        metadata: vector<u8>,
    }

    /// Message for accept token events
    struct AcceptTokenEvent {
        token_code: vector<u8>,
    }

    fun EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED(): u64 { ErrorCode::ECODE_BASE() + 1}
    fun EMALFORMED_AUTHENTICATION_KEY(): u64 { ErrorCode::ECODE_BASE() + 2}
    fun EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED(): u64 { ErrorCode::ECODE_BASE() + 3}
    fun ADDRESS_PUBLIC_KEY_INCONSISTENT(): u64 { ErrorCode::ECODE_BASE() + 4}

    const DUMMY_AUTH_KEY:vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";

    // Create an genesis account at `new_account_address` and return signer.
    // Genesis authentication_key is zero bytes.
    public fun create_genesis_account(
        new_account_address: address,
    ) :signer {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        let new_account = create_signer(new_account_address);
        make_account(&new_account, DUMMY_AUTH_KEY);
        new_account
    }

    spec fun create_genesis_account {
        aborts_if !Timestamp::is_genesis();
        aborts_if len(DUMMY_AUTH_KEY) != 32;
        aborts_if exists<Account>(new_account_address);
    }

    // Release genesis account signer
    public fun release_genesis_signer(genesis_account: signer){
        destroy_signer(genesis_account);
    }

    spec fun release_genesis_signer {
        aborts_if false;
    }

    // Creates a new account at `fresh_address` with a balance of zero and public
    // key `public_key_vec` | `fresh_address`.
    // Creating an account at address 0x1 will cause runtime failure as it is a
    // reserved address for the MoveVM.
    public fun create_account<TokenType>(fresh_address: address, public_key_vec: vector<u8>) acquires Account {
        let authentication_key = Authenticator::ed25519_authentication_key(public_key_vec);
        let new_address = Authenticator::derived_address(copy authentication_key);
        assert(new_address == fresh_address, ADDRESS_PUBLIC_KEY_INCONSISTENT());

        let new_account = create_signer(new_address);
        make_account(&new_account, authentication_key);
        // Make sure all account accept STC.
        if (!STC::is_stc<TokenType>()){
            accept_token<STC>(&new_account);
        };
        accept_token<TokenType>(&new_account);
        destroy_signer(new_account);
    }

    spec fun create_account {
        pragma verify = false;
        // missing spec_token_code()
        //aborts_if token_code<TokenType>() != token_code<STC>() && exists<Balance<TokenType>>(fresh_address);
        aborts_if len(public_key_vec) != 32;
        aborts_if exists<Account>(fresh_address);
    }

    fun make_account(
        new_account: &signer,
        authentication_key: vector<u8>,
    ) {
        assert(Vector::length(&authentication_key) == 32, EMALFORMED_AUTHENTICATION_KEY());
        let new_account_addr = Signer::address_of(new_account);
        Event::publish_generator(new_account);
        move_to(new_account, Account {
              authentication_key,
              withdrawal_capability: Option::some(
                  WithdrawCapability {
                      account_address: new_account_addr
              }),
              key_rotation_capability: Option::some(
                  KeyRotationCapability {
                      account_address: new_account_addr
              }),
              received_events: Event::new_event_handle<ReceivedPaymentEvent>(new_account),
              sent_events: Event::new_event_handle<SentPaymentEvent>(new_account),
              accept_token_events: Event::new_event_handle<AcceptTokenEvent>(new_account),
              sequence_number: 0,
        });
    }

    spec fun make_account {
        aborts_if len(authentication_key) != 32;
        aborts_if exists<Account>(Signer::address_of(new_account));
    }

    native fun create_signer(addr: address): signer;
    native fun destroy_signer(sig: signer);

    // Deposits the `to_deposit` token into the `payee`'s account balance
    public fun deposit_to<TokenType>(account: &signer, payee: address, to_deposit: Token<TokenType>)
    acquires Account, Balance {
        deposit_with_metadata(account, payee, to_deposit, x"")
    }

    spec fun deposit_to {
        include Deposit_With_Payer_And_Metadata<TokenType>{payer: Signer::spec_address_of(account)};
    }

    // Deposits the `to_deposit` token into the account balance
    public fun deposit<TokenType>(account: &signer, to_deposit: Token<TokenType>)
    acquires Account, Balance {
        let account_address = Signer::address_of(account);
        if (!is_accepts_token<TokenType>(account_address)){
            accept_token<TokenType>(account);
        };
        deposit_to(account, account_address, to_deposit)
    }

    spec fun deposit {
        pragma verify = false; //Todo: fix me
        aborts_if to_deposit.value == 0;
        aborts_if !exists<Account>(Signer::address_of(account));
        aborts_if global<Balance<TokenType>>(Signer::address_of(account)).token.value + to_deposit.value > max_u128();
        ensures global<Balance<TokenType>>(Signer::address_of(account)).token.value == old(global<Balance<TokenType>>(Signer::address_of(account)).token.value) + to_deposit.value;

    }

    // Deposits the `to_deposit` token into the `payee`'s account balance with the attached `metadata`
    public fun deposit_with_metadata<TokenType>(account: &signer,
        payee: address,
        to_deposit: Token<TokenType>,
        metadata: vector<u8>,
    ) acquires Account, Balance {
        deposit_with_payer_and_metadata(
            Signer::address_of(account),
            payee,
            to_deposit,
            metadata,
        );
    }

    spec fun deposit_with_metadata {
        include Deposit_With_Payer_And_Metadata<TokenType>{payer: Signer::spec_address_of(account)};
    }

    // Deposits the `to_deposit` token into the `payee`'s account balance with the attached `metadata` and
    // payer address
    fun deposit_with_payer_and_metadata<TokenType>(
        payer: address,
        payee: address,
        to_deposit: Token<TokenType>,
        metadata: vector<u8>,
    ) acquires Account, Balance {
        // Check that the `to_deposit` token is non-zero
        let deposit_value = Token::value(&to_deposit);
        assert(deposit_value > 0, ErrorCode::ECOIN_DEPOSIT_IS_ZERO());

        let token_code = Token::token_code<TokenType>();

        // Load the payer's account
        let payer_account_ref = borrow_global_mut<Account>(payer);
        // Log a sent event
        Event::emit_event<SentPaymentEvent>(
            &mut payer_account_ref.sent_events,
            SentPaymentEvent {
                amount: deposit_value,
                token_code: (copy token_code),
                payee: payee,
                metadata: *&metadata
            },
        );

        // Load the payee's account
        let payee_account_ref = borrow_global_mut<Account>(payee);
        let payee_balance = borrow_global_mut<Balance<TokenType>>(payee);
        // Deposit the `to_deposit` token
        Token::deposit(&mut payee_balance.token, to_deposit);
        // Log a received event
        Event::emit_event<ReceivedPaymentEvent>(
            &mut payee_account_ref.received_events,
            ReceivedPaymentEvent {
                amount: deposit_value,
                token_code: token_code,
                payer: payer,
                metadata: metadata
            }
        );
    }

    spec fun deposit_with_payer_and_metadata {
        include Deposit_With_Payer_And_Metadata<TokenType>;
    }

    spec schema Deposit_With_Payer_And_Metadata<TokenType> {
        payer: address;
        payee: address;
        to_deposit: Token<TokenType>;

        aborts_if to_deposit.value == 0;
        aborts_if !exists<Account>(payer);
        aborts_if !exists<Account>(payee);
        aborts_if !exists<Balance<TokenType>>(payee);
        aborts_if global<Balance<TokenType>>(payee).token.value + to_deposit.value > max_u128();
        ensures global<Balance<TokenType>>(payee).token.value == old(global<Balance<TokenType>>(payee).token.value) + to_deposit.value;
    }

    // Helper to withdraw `amount` from the given account balance and return the withdrawn Token<TokenType>
    fun withdraw_from_balance<TokenType>(_addr: address, balance: &mut Balance<TokenType>, amount: u128): Token<TokenType>{
        Token::withdraw(&mut balance.token, amount)
    }

    spec fun withdraw_from_balance {
        aborts_if balance.token.value < amount;
        ensures balance.token.value == old(balance.token.value) - amount;
    }

    // Withdraw `amount` Token<TokenType> from the account balance
    public fun withdraw<TokenType>(account: &signer, amount: u128): Token<TokenType>
    acquires Account, Balance {
        let sender_addr = Signer::address_of(account);
        let sender_balance = borrow_global_mut<Balance<TokenType>>(sender_addr);
        // The sender_addr has delegated the privilege to withdraw from her account elsewhere--abort.
        assert(!delegated_withdraw_capability(sender_addr), EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED());
        // The sender_addr has retained her withdrawal privileges--proceed.
        withdraw_from_balance<TokenType>(sender_addr, sender_balance, amount)
    }

    spec fun withdraw {
        aborts_if Option::spec_is_none(global<Account>(Signer::address_of(account)).withdrawal_capability);
        aborts_if !exists<Account>(Signer::address_of(account));
        aborts_if !exists<Balance<TokenType>>(Signer::address_of(account));
        aborts_if global<Balance<TokenType>>(Signer::address_of(account)).token.value < amount;
        ensures global<Balance<TokenType>>(Signer::address_of(account)).token.value
                == old(global<Balance<TokenType>>(Signer::address_of(account)).token.value) - amount;
    }

    // Withdraw `amount` Token<TokenType> from the account under cap.account_address
    public fun withdraw_with_capability<TokenType>(
        cap: &WithdrawCapability, amount: u128
    ): Token<TokenType> acquires Balance {
        let balance = borrow_global_mut<Balance<TokenType>>(cap.account_address);
        withdraw_from_balance<TokenType>(cap.account_address, balance , amount)
    }

    spec fun withdraw_with_capability {
        aborts_if !exists<Balance<TokenType>>(cap.account_address);
        aborts_if global<Balance<TokenType>>(cap.account_address).token.value < amount;
        ensures global<Balance<TokenType>>(cap.account_address).token.value
                == old(global<Balance<TokenType>>(cap.account_address).token.value) - amount;
    }

    // Return a unique capability granting permission to withdraw from the sender's account balance.
    public fun extract_withdraw_capability(
        sender: &signer
    ): WithdrawCapability acquires Account {
        let sender_addr = Signer::address_of(sender);
        // Abort if we already extracted the unique withdraw capability for this account.
        assert(!delegated_withdraw_capability(sender_addr), EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED());
        let account = borrow_global_mut<Account>(sender_addr);
        Option::extract(&mut account.withdrawal_capability)
    }

    spec fun extract_withdraw_capability {
        aborts_if !exists<Account>(Signer::address_of(sender));
        aborts_if Option::spec_is_none(global<Account>( Signer::spec_address_of(sender)).withdrawal_capability);
    }

     // Return the withdraw capability to the account it originally came from
     public fun restore_withdraw_capability(cap: WithdrawCapability)
        acquires Account {
            let account = borrow_global_mut<Account>(cap.account_address);
            Option::fill(&mut account.withdrawal_capability, cap)
     }

    spec fun restore_withdraw_capability {
        aborts_if Option::spec_is_some(global<Account>(cap.account_address).withdrawal_capability);
        aborts_if !exists<Account>(cap.account_address);
    }

    // Withdraws `amount` Token<TokenType> using the passed in WithdrawCapability, and deposits it
    // into the `payee`'s account balance. Creates the `payee` account if it doesn't exist.
    public fun pay_from_capability<TokenType>(
        cap: &WithdrawCapability,
        payee: address,
        amount: u128,
        metadata: vector<u8>,
    ) acquires Account, Balance {
        deposit_with_payer_and_metadata<TokenType>(
            *&cap.account_address,
            payee,
            withdraw_with_capability(cap, amount),
            metadata,
        );
    }

    spec fun pay_from_capability {
        pragma verify = false; //Todo: fix me

        aborts_if !exists<Balance<TokenType>>(cap.account_address);
        aborts_if global<Balance<TokenType>>(cap.account_address).token.value < amount;


        aborts_if amount == 0;
        aborts_if !exists<Account>(cap.account_address);
        aborts_if !exists<Account>(payee);
        aborts_if !exists<Balance<TokenType>>(payee);
        //aborts_if global<Balance<TokenType>>(payee).token.value + amount > max_u128();
        //ensures global<Balance<TokenType>>(payee).token.value == old(global<Balance<TokenType>>(payee).token.value) + amount;

    }

    // Withdraw `amount` Token<TokenType> from the transaction sender's
    // account balance and send the token to the `payee` address with the
    // attached `metadata` Creates the `payee` account if it does not exist
    public fun pay_from_with_metadata<TokenType>(
        account: &signer,
        payee: address,
        amount: u128,
        metadata: vector<u8>,
    ) acquires Account, Balance {
        deposit_with_metadata<TokenType>(
            account,
            payee,
            withdraw(account, amount),
            metadata,
        );
    }

    spec fun pay_from_with_metadata {
        pragma verify = false;

        aborts_if !exists<Balance<TokenType>>(Signer::address_of(account));
        aborts_if global<Balance<TokenType>>(Signer::address_of(account)).token.value < amount;
        ensures global<Balance<TokenType>>(Signer::address_of(account)).token.value
                == old(global<Balance<TokenType>>(Signer::address_of(account)).token.value) - amount;

        include Deposit_With_Payer_And_Metadata<TokenType>{
            payer: Signer::address_of(account),
            to_deposit: Token<TokenType> { value: amount }
        };
    }

    // Withdraw `amount` Token<TokenType> from the transaction sender's
    // account balance  and send the token to the `payee` address
    // Creates the `payee` account if it does not exist
    public fun pay_from<TokenType>(
        account: &signer,
        payee: address,
        amount: u128
    ) acquires Account, Balance {
        pay_from_with_metadata<TokenType>(account, payee, amount, x"");
    }

    spec fun pay_from {
        pragma verify = false;

        aborts_if !exists<Balance<TokenType>>(Signer::address_of(account));
        aborts_if global<Balance<TokenType>>(Signer::address_of(account)).token.value < amount;
        ensures global<Balance<TokenType>>(Signer::address_of(account)).token.value
                == old(global<Balance<TokenType>>(Signer::address_of(account)).token.value) - amount;

        include Deposit_With_Payer_And_Metadata<TokenType>{
            payer: Signer::address_of(account),
            to_deposit: Token<TokenType> { value: amount }
        };
    }

    // Rotate the authentication key for the account under cap.account_address
    public fun rotate_authentication_key(
        cap: &KeyRotationCapability,
        new_authentication_key: vector<u8>,
    ) acquires Account  {
        let sender_account_resource = borrow_global_mut<Account>(cap.account_address);
        // Don't allow rotating to clearly invalid key
        assert(Vector::length(&new_authentication_key) == 32, EMALFORMED_AUTHENTICATION_KEY());
        sender_account_resource.authentication_key = new_authentication_key;
    }

    spec fun rotate_authentication_key {
        aborts_if !exists<Account>(cap.account_address);
        aborts_if len(new_authentication_key) != 32;
        ensures global<Account>(cap.account_address).authentication_key == new_authentication_key;
    }
    spec module {
        define spec_rotate_authentication_key(addr: address, new_authentication_key: vector<u8>): bool {
            global<Account>(addr).authentication_key == new_authentication_key
        }
    }

    // Return a unique capability granting permission to rotate the sender's authentication key
    public fun extract_key_rotation_capability(account: &signer): KeyRotationCapability
    acquires Account {
        let account_address = Signer::address_of(account);
        // Abort if we already extracted the unique key rotation capability for this account.
        assert(!delegated_key_rotation_capability(account_address), EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED());
        let account = borrow_global_mut<Account>(account_address);
        Option::extract(&mut account.key_rotation_capability)
    }

    spec fun extract_key_rotation_capability {
        aborts_if !exists<Account>(Signer::address_of(account));
        aborts_if Option::spec_is_none(global<Account>(Signer::spec_address_of(account)).key_rotation_capability);
    }

    // Return the key rotation capability to the account it originally came from
    public fun restore_key_rotation_capability(cap: KeyRotationCapability)
    acquires Account {
        let account = borrow_global_mut<Account>(cap.account_address);
        Option::fill(&mut account.key_rotation_capability, cap)
    }

    spec fun restore_key_rotation_capability {
        aborts_if Option::spec_is_some(global<Account>(cap.account_address).key_rotation_capability);
        aborts_if !exists<Account>(cap.account_address);
    }

    // Helper to return the u128 value of the `balance` for `account`
    fun balance_for<TokenType>(balance: &Balance<TokenType>): u128 {
        Token::value<TokenType>(&balance.token)
    }

    spec fun balance_for {
        aborts_if false;
    }

    // Return the current TokenType balance of the account at `addr`.
    public fun balance<TokenType>(addr: address): u128 acquires Balance {
        balance_for(borrow_global<Balance<TokenType>>(addr))
    }

    spec fun balance {
        aborts_if !exists<Balance<TokenType>>(addr);
    }

    // Add a balance of `Token` type to the sending account.
    public fun accept_token<TokenType>(account: &signer) acquires Account {
        move_to(account, Balance<TokenType>{ token: Token::zero<TokenType>() });
        let token_code = Token::token_code<TokenType>();
        // Load the sender's account
        let sender_account_ref = borrow_global_mut<Account>(Signer::address_of(account));
        // Log a sent event
        Event::emit_event<AcceptTokenEvent>(
            &mut sender_account_ref.accept_token_events,
            AcceptTokenEvent {
                token_code:  token_code,
            },
        );
    }

    spec fun accept_token {
        aborts_if exists<Balance<TokenType>>(Signer::address_of(account));
        aborts_if !exists<Account>(Signer::address_of(account));

    }

    // Return whether the account at `addr` accepts `Token` type tokens
    public fun is_accepts_token<TokenType>(addr: address): bool {
        exists<Balance<TokenType>>(addr)
    }

    spec fun is_accepts_token {
        aborts_if false;
    }

    // Helper to return the sequence number field for given `account`
    fun sequence_number_for_account(account: &Account): u64 {
        account.sequence_number
    }

    spec fun is_accepts_token {
        aborts_if false;
    }

    // Return the current sequence number at `addr`
    public fun sequence_number(addr: address): u64 acquires Account {
        sequence_number_for_account(borrow_global<Account>(addr))
    }

    spec fun sequence_number {
        aborts_if !exists<Account>(addr);
    }

    // Return the authentication key for this account
    public fun authentication_key(addr: address): vector<u8> acquires Account {
        *&borrow_global<Account>(addr).authentication_key
    }

    spec fun authentication_key {
        aborts_if !exists<Account>(addr);
    }

    // Return true if the account at `addr` has delegated its key rotation capability
    public fun delegated_key_rotation_capability(addr: address): bool
    acquires Account {
        Option::is_none(&borrow_global<Account>(addr).key_rotation_capability)
    }

    spec fun delegated_key_rotation_capability {
        aborts_if !exists<Account>(addr);
    }

    // Return true if the account at `addr` has delegated its withdraw capability
    public fun delegated_withdraw_capability(addr: address): bool
    acquires Account {
        Option::is_none(&borrow_global<Account>(addr).withdrawal_capability)
    }

    spec fun delegated_withdraw_capability {
        aborts_if !exists<Account>(addr);
    }

    // Return a reference to the address associated with the given withdraw capability
    public fun withdraw_capability_address(cap: &WithdrawCapability): &address {
        &cap.account_address
    }

    spec fun withdraw_capability_address {
        aborts_if false;
    }

    // Return a reference to the address associated with the given key rotation capability
    public fun key_rotation_capability_address(cap: &KeyRotationCapability): &address {
        &cap.account_address
    }

    spec fun key_rotation_capability_address {
        aborts_if false;
    }

    // Checks if an account exists at `check_addr`
    public fun exists_at(check_addr: address): bool {
        exists<Account>(check_addr)
    }

    spec fun exists_at {
        aborts_if false;
    }

    // The prologue is invoked at the beginning of every transaction
    // It verifies:
    // - The account's auth key matches the transaction's public key
    // - That the account has enough balance to pay for all of the gas
    // - That the sequence number matches the transaction's sequence key
    public fun txn_prologue<TokenType>(
        account: &signer,
        txn_sender: address,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
    ) acquires Account, Balance {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST());

        // FUTURE: Make these error codes sequential
        // Verify that the transaction sender's account exists
        assert(exists_at(txn_sender), ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST());

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<Account>(txn_sender);

        // Check that the hash of the transaction's public key matches the account's auth key
        assert(
            Hash::sha3_256(txn_public_key) == *&sender_account.authentication_key,
            ErrorCode::PROLOGUE_INVALID_ACCOUNT_AUTH_KEY()
        );

        // Check that the account has enough balance for all of the gas
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        let balance_amount = balance<TokenType>(txn_sender);
        assert(balance_amount >= (max_transaction_fee as u128), ErrorCode::PROLOGUE_CANT_PAY_GAS_DEPOSIT());

        // Check that the transaction sequence number matches the sequence number of the account
        assert(txn_sequence_number >= sender_account.sequence_number, ErrorCode::PROLOGUE_SEQUENCE_NUMBER_TOO_OLD());
        assert(txn_sequence_number == sender_account.sequence_number, ErrorCode::PROLOGUE_SEQUENCE_NUMBER_TOO_NEW());
    }

    spec fun txn_prologue {
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<Account>(txn_sender);
        aborts_if Hash::sha3_256(txn_public_key) != global<Account>(txn_sender).authentication_key;
        aborts_if txn_gas_price * txn_max_gas_units > max_u64();
        aborts_if !exists<Balance<TokenType>>(txn_sender);
        aborts_if global<Balance<TokenType>>(txn_sender).token.value < txn_gas_price * txn_max_gas_units;
        aborts_if txn_sequence_number < global<Account>(txn_sender).sequence_number;
        aborts_if txn_sequence_number != global<Account>(txn_sender).sequence_number;
    }

    // The epilogue is invoked at the end of transactions.
    // It collects gas and bumps the sequence number
    public fun txn_epilogue<TokenType>(
        account: &signer,
        txn_sender: address,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64,
    ) acquires Account, Balance {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        // Load the transaction sender's account and balance resources
        let sender_account = borrow_global_mut<Account>(txn_sender);
        let sender_balance = borrow_global_mut<Balance<TokenType>>(txn_sender);

        // Charge for gas
        let transaction_fee_amount =(txn_gas_price * (txn_max_gas_units - gas_units_remaining) as u128);
        assert(
            balance_for(sender_balance) >= transaction_fee_amount,
            ErrorCode::EINSUFFICIENT_BALANCE()
        );

        // Bump the sequence number
        sender_account.sequence_number = txn_sequence_number + 1;

        if (transaction_fee_amount > 0) {
            let transaction_fee = withdraw_from_balance(
                    Signer::address_of(account),
                    sender_balance,
                    transaction_fee_amount
            );
            TransactionFee::pay_fee(transaction_fee);
        };
    }

    spec fun txn_epilogue {
        pragma verify = false; // Todo: fix me, cost too much time
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<Account>(txn_sender);
        aborts_if !exists<Balance<TokenType>>(txn_sender);
        aborts_if txn_gas_price * (txn_max_gas_units - gas_units_remaining) > max_u64();
        aborts_if txn_max_gas_units < gas_units_remaining;
        aborts_if global<Balance<TokenType>>(txn_sender).token.value < txn_gas_price * (txn_max_gas_units - gas_units_remaining);
        aborts_if txn_sequence_number + 1 > max_u64();
        aborts_if txn_gas_price * (txn_max_gas_units - gas_units_remaining) > 0 &&
                   !exists<TransactionFee::TransactionFee<TokenType>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if global<TransactionFee::TransactionFee<TokenType>>(CoreAddresses::SPEC_GENESIS_ADDRESS()).fee.value + txn_gas_price * (txn_max_gas_units - gas_units_remaining) > max_u128();

    }
}

}
