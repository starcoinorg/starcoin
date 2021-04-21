// Test the treasury withdraw.
//! account: alice, 0 0x1::STC::STC

//! sender: association
script {
    use 0x1::STC::STC;
    use 0x1::Treasury;
    //use 0x1::Debug;

    fun mint(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        assert(Treasury::get_linear_withdraw_capability_total(&cap) == 47777040000000000, 1000);
        assert(Treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        assert(Treasury::get_linear_withdraw_capability_start_time(&cap) == 0, 1002);
        assert(Treasury::get_linear_withdraw_capability_period(&cap) ==94608000, 1003);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//! block-prologue
//! author: alice
//! block-time: 3600000
//! block-number: 1

//! new-transaction
//! sender: association
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Treasury;
    use 0x1::Token;
    use 0x1::Debug;

    fun mint(account: signer) {
        let linear_cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = Treasury::withdraw_with_linear_cap(&mut linear_cap);
        Debug::print(&Token::value(&token));
        assert(Token::value(&token) == 1818000000000, 1004);
        Treasury::add_linear_withdraw_capability(&account, linear_cap);
        Account::deposit_to_self(&account, token);
    }
}

// check: EXECUTED

//! block-prologue
//! author: alice
//! block-time: 7200000
//! block-number: 2

//! new-transaction
//! sender: association
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Treasury;
    use 0x1::Token;
    //use 0x1::Debug;

    fun mint(account: signer) {
        let linear_cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = Treasury::withdraw_with_linear_cap(&mut linear_cap);
        //Debug::print(&Token::value(&token));
        assert(Token::value(&token) == 1818000000000, 1005);
        Treasury::add_linear_withdraw_capability(&account, linear_cap);
        Account::deposit_to_self(&account, token);
    }
}

// check: EXECUTED

//! block-prologue
//! author: alice
//! block-time: 94608000000
//! block-number: 3

//! new-transaction
//! sender: association
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Treasury;
    use 0x1::Token;
    //use 0x1::Debug;

    fun mint(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = Treasury::withdraw_with_linear_cap(&mut cap);
        //Debug::print(&Token::value(&token));
        assert(Token::value(&token) == (47777040000000000 - 1818000000000*2), 1006);
        Account::deposit_to_self(&account, token);
        assert(Treasury::get_linear_withdraw_capability_withdraw(&cap) == Treasury::get_linear_withdraw_capability_total(&cap), 1007);
        Treasury::destroy_linear_withdraw_capability(cap);
    }
}