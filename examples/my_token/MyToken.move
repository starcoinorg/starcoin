module {{sender}}::MyToken {
     use StarcoinFramework::Token;
     use StarcoinFramework::Account;

     struct MyToken has copy, drop, store { }

     public(script) fun init(account: signer) {
         Token::register_token<MyToken>(&account, 3);
         Account::do_accept_token<MyToken>(&account);
     }

     public(script) fun mint(account: signer, amount: u128) {
        let token = Token::mint<MyToken>(&account, amount);
        Account::deposit_to_self<MyToken>(&account, token)
     }
}
