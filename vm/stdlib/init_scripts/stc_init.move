script {
use 0x1::STC::{Self, STC};
use 0x1::Account;

fun stc_init(association: &signer) {
    STC::initialize(association);
    Account::add_currency<STC>(association);
}
}