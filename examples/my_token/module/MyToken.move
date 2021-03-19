module MyToken {
     use 0x1::Token;
     use 0x1::Account;

     struct MyToken has copy, drop, store { }

     public fun init(account: &signer) {
         Token::register_token<MyToken>(account, 3);
         Account::do_accept_token<MyToken>(account);
     }

     public fun mint(account: &signer, amount: u128) {
        let token = Token::mint<MyToken>(account, amount);
        Account::deposit_to_self<MyToken>(account, token)
     }
}
