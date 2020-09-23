script {
use 0x1::TransactionPublishOption;

/// Append the `hash` to script hashes list allowed to be executed by the network.
/// Todo: it's dangous to run the script when publish option is VMPublishingOption::Open
/// because the list is empty at the moment, adding script into the empty list will lead to
/// that only the added script is allowed to execute.
fun add_to_script_allow_list(account: &signer, hash: vector<u8>) {
    TransactionPublishOption::add_to_script_allow_list(account, hash)
}
}
