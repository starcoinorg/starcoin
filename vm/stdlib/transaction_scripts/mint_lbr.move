script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::Account;
use 0x0::Transaction;
fun main(amount_lbr: u64) {
    let sender = Transaction::sender();
    let coin1_balance = Account::balance<Coin1::T>(sender);
    let coin2_balance = Account::balance<Coin2::T>(sender);
    let coin1 = Account::withdraw_from_sender<Coin1::T>(coin1_balance);
    let coin2 = Account::withdraw_from_sender<Coin2::T>(coin2_balance);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    Account::deposit(sender, lbr);
    Account::deposit(sender, coin1);
    Account::deposit(sender, coin2);
}
}
