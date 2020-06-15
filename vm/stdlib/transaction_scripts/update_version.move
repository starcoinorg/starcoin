script {
use 0x1::Version;

fun main(account: &signer,major: u64) {
    Version::set(account, major)
}
}
