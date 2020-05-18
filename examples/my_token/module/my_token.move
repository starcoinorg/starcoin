address 0xeae6b71b9583150c1c32bc9500ee5d15:

module MyToken {
     use 0x0::Libra;
     use 0x0::Account;
     use 0x0::Transaction;

     struct T { }

     public fun issue(amount: u64) {
         // only specific address can issue the token
         Transaction::assert(Transaction::sender() == 0xeae6b71b9583150c1c32bc9500ee5d15, 8000);

         // register token
         Libra::register<T>(T{});

         // mint 'amount' tokens
         let coin = Libra::mint<T>(amount);

         // create 'Balance<Token>' resource under sender account
         Account::create_new_balance<T>();

         // deposit tokens into sender's Balance resource
         Account::deposit_to_sender<T>(coin)
     }
 }