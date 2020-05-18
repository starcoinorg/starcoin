script {
use 0x0::LBR;
use 0x0::Account;
use 0x0::Transaction;
fun main(amount_lbr: u64) {
    let sender = Transaction::sender();
    let lbr = Account::withdraw_from_sender<LBR::T>(amount_lbr);
    let (coin1, coin2) = LBR::unpack(lbr);
    Account::deposit(sender, coin1);
    Account::deposit(sender, coin2);
}
}
