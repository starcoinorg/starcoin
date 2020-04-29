address 0xeae6b71b9583150c1c32bc9500ee5d15:

module MyToken {
     use 0x0::Libra;
     use 0x0::LibraAccount;

     struct T { }

     public fun issue(amount: u64) {
         // register token
         Libra::register<T>();

         // mint 'amount' tokens and check that the market cap increases appropriately
         let coin = Libra::mint<T>(amount);

         // create 'Balance<Token>' resource under sender account
         LibraAccount::create_new_balance<T>();

         // deposit tokens into sender's Balance resource
         LibraAccount::deposit_to_sender<T>(coin)
     }
 }