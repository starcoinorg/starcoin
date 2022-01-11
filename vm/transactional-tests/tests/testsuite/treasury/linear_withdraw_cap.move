//# init -n dev

//# faucet --addr alice --amount 0

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury;
    //use StarcoinFramework::Debug;

    fun mint(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        assert!(Treasury::get_linear_withdraw_capability_total(&cap) == 477770400000000000, 1000);
        assert!(Treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        assert!(Treasury::get_linear_withdraw_capability_start_time(&cap) == 0, 1002);
        StarcoinFramework::Debug::print(&Treasury::get_linear_withdraw_capability_period(&cap));
        assert!(Treasury::get_linear_withdraw_capability_period(&cap) ==86400, 1003);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//# block --author alice --timestamp 3600000

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury;
    use StarcoinFramework::Token;
    use StarcoinFramework::Debug;

    fun mint(account: signer) {
        let linear_cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = Treasury::withdraw_with_linear_capability(&mut linear_cap);
        Debug::print(&Token::value(&token));
        assert!(Token::value(&token) == 19907100000000000, 1004);
        Treasury::add_linear_withdraw_capability(&account, linear_cap);
        Account::deposit_to_self(&account, token);
    }
}


//# block --author alice --timestamp 7200000

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury;
    use StarcoinFramework::Token;
    //use StarcoinFramework::Debug;

    fun mint(account: signer) {
        let linear_cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = Treasury::withdraw_with_linear_capability(&mut linear_cap);
        StarcoinFramework::Debug::print(&Token::value(&token));
        assert!(Token::value(&token) == 19907100000000000, 1005);
        Treasury::add_linear_withdraw_capability(&account, linear_cap);
        Account::deposit_to_self(&account, token);
    }
}


//# block --author alice --timestamp 94608000000

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury;
    use StarcoinFramework::Token;
    //use StarcoinFramework::Debug;

    fun mint(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = Treasury::withdraw_with_linear_capability(&mut cap);
        StarcoinFramework::Debug::print(&Token::value(&token));
        assert!(Token::value(&token) == (477770400000000000 - 19907100000000000*2), 1006);
        Account::deposit_to_self(&account, token);
        assert!(Treasury::get_linear_withdraw_capability_withdraw(&cap) == Treasury::get_linear_withdraw_capability_total(&cap), 1007);
        Treasury::destroy_linear_withdraw_capability(cap);
    }
}