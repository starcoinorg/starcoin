script {
use 0x0::Coin;
use 0x0::Account;

// Preburn `amount` `Token`s from the sender's account.
// This will only succeed if the sender already has a published `Preburn<Token>` resource.
fun main<Token>(amount: u64) {
    Coin::preburn_to_sender<Token>(Account::withdraw_from_sender(amount))
}
}
