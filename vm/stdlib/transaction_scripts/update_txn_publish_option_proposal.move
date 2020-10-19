script {
use 0x1::TransactionPublishOption;
use 0x1::OnChainConfigDao;
use 0x1::STC;

fun update_txn_publish_option_proposal(account: &signer,
    script_allow_list: vector<u8>,
    module_publishing_allowed: bool,
    exec_delay: u64) {
    let txn_publish_option = TransactionPublishOption::new_transaction_publish_option(script_allow_list, module_publishing_allowed);
    OnChainConfigDao::propose_update<STC::STC, TransactionPublishOption::TransactionPublishOption>(account, txn_publish_option, exec_delay);
}

spec fun update_txn_publish_option_proposal {
    pragma verify = false;
}
}
