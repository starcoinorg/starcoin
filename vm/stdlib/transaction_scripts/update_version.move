script {
use 0x0::Version;

fun main(account: &signer,major: u64) {
    Version::set(account, major)
}
}
