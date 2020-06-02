script {
use 0x0::VMConfig;

fun main(account: &signer, args: vector<u8>) {
    VMConfig::set_publishing_option(account, args)
}
}
