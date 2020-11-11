//! account: alice

//! new-transaction
//! sender: alice
script{
use 0x1::Version;
use 0x1::Config;
fun main(account: &signer) {
    Config::publish_new_config<Version::Version>(account, Version::new_version(1));
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script{
use 0x1::Version;
use 0x1::Signer;
fun main(account: &signer) {
    let version = Version::get(Signer::address_of(account));
    assert(version == 1, 100);
    let _ = version;
}
}

// check: EXECUTED

