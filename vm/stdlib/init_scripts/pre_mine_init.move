script {
use 0x1::Account;
use 0x1::STC::STC;
use 0x1::CoreAddresses;
use 0x1::Signer;

fun pre_mine_init(association: &signer, total_supply: u64, pre_mine_percent:u64) {
    let association_balance = total_supply * pre_mine_percent / 100;
    if (association_balance > 0) {
        Account::mint_to_address<STC>(association, Signer::address_of(association), association_balance);
    };
    let miner_reward_balance = total_supply + association_balance;
    Account::mint_to_address<STC>(association, CoreAddresses::MINT_ADDRESS(), miner_reward_balance);
}
}