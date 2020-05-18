script {
use 0x0::ValidatorConfig;
use 0x0::System;

fun main (new_key: vector<u8>) {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(new_key);
    System::update_and_reconfigure();
}
}
