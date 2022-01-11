//# init -n dev

//# faucet --addr alice

//# run --signers alice
script{
use StarcoinFramework::Version;
use StarcoinFramework::Config;
fun main(account: signer) {
    Config::publish_new_config<Version::Version>(&account, Version::new_version(1));
}
}

//# run --signers alice
script{
use StarcoinFramework::Version;
use StarcoinFramework::Signer;
fun main(account: signer) {
    let version = Version::get(Signer::address_of(&account));
    assert!(version == 1, 100);
    let _ = version;
}
}

