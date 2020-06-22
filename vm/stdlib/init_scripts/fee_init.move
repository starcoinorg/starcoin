script {
use 0x1::Signer;
use 0x1::STC::STC;
use 0x1::Account;
use 0x1::CoreAddresses;

fun stc_init(fee_account: &signer) {
    assert(Signer::address_of(fee_account) == CoreAddresses::TRANSACTION_FEE_ADDRESS(),1);
    Account::add_currency<STC>(fee_account);
}
}