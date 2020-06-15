script {
use 0x1::VMConfig;

fun main(account: &signer, args: vector<u8>) {
    VMConfig::set_publishing_option(account, args)
}
}
