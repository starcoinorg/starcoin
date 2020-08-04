module MyToken {
     use 0x1::Token;
     use 0x1::Account;

     struct MyToken { }

     public fun init(account: &signer) {
         Token::register_token<MyToken>(
                     account,
                     1000000, // scaling_factor = 10^6
                     1000,    // fractional_part = 10^3
         );
         Account::accept_token<MyToken>(account);
     }

     public fun mint(account: &signer, amount: u128) {
        let token = Token::mint<MyToken>(account, amount);
        Account::deposit<MyToken>(account, token)
     }
}
