script {
use 0x1::TransactionPublishOption;

/// Append the `hash` to script hashes list allowed to be executed by the network.
fun add_to_script_allow_list(account: &signer, hash: vector<u8>) {
    TransactionPublishOption::add_to_script_allow_list(account, hash)
}
}
