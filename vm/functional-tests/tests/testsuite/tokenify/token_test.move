//! account: alice
//! account: bob

//! new-transaction
//! sender: alice

  module Coin1 {
      use 0x1::Signer;
      use 0x1::Token;

      resource struct T {}

      /// just for test, can be generic of `CoinType`
      resource struct Balance {
         coin: Token::Coin<T>,
      }

      public fun register(signer: &signer)
      acquires T {
          assert(Signer::address_of(signer) == token_address(), 42);
          let token = T {};
          move_to(signer, token);
          let borrowed_token = borrow_global<T>(Signer::address_of(signer));
          Token::register_currency(signer, borrowed_token, 1000, 1000);
      }

      public fun mint_to(signer: &signer, amount: u64, receiver: address)
      acquires Balance {

          let coin = Token::mint<T>(signer, amount, token_address());
          let receiver_balance_ref_mut = borrow_global_mut<Balance>(receiver);
          Token::deposit(&mut receiver_balance_ref_mut.coin, coin);
      }

      public fun accept(signer: &signer) {
          let zero_coin = Token::zero<T>();
          let b = Balance {
            coin: zero_coin,
          };
          move_to(signer, b)
      }

      public fun balance(signer: &signer): u64 acquires Balance {
        let balance_ref = borrow_global<Balance>(Signer::address_of(signer));
        Token::value(&balance_ref.coin)
      }

      /// TODO
      /// many other function can be implemented based on `Balance`, like `transfer`, `mint`, `withdraw`


      public fun token_address(): address {
          {{alice}}
      }
  }

// check: EXECUTED


//! new-transaction
//! sender: alice
script {
  use {{alice}}::Coin1;
  fun register_token(signer: &signer) {
    Coin1::register(signer);
    Coin1::accept(signer);
  }
}

// check: EXECUTED

//! new-transaction
//! sender: alice

script {
  use {{alice}}::Coin1;
  use 0x1::Signer;
  fun mint_coin(signer: &signer) {
    Coin1::mint_to(signer, 10000, Signer::address_of(signer));

    let balance = Coin1::balance(signer);
    assert(balance == 10000, 10000);
  }
}
// check: EXECUTED

//! new-transaction
//! sender: bob

script {
  use {{alice}}::Coin1;
  fun register_token(signer: &signer) {
    Coin1::register(signer);
    Coin1::accept(signer);
  }
}
// check: ABORTED