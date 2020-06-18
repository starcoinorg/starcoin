script {
use 0x1::Block;
use 0x1::STC::STC;
use 0x1::Account;

fun mint_init(mint_account: &signer) {
    Account::add_currency<STC>(mint_account);
    Block::initialize_reward_info(mint_account);
}
}