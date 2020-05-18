script {
use 0x0::VMConfig;

fun main(args: vector<u8>) {
    VMConfig::set_publishing_option(args)
}
}
