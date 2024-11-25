//# init -n dev

//# faucet --addr alice

//# run --signers alice
script{
use starcoin_framework::stc_version::Version;
use starcoin_framework::on_chain_config;
fun main(account: signer) {
    Config::publish_new_config<Version::Version>(&account, Version::new_version(1));
}
}

//# run --signers alice
script{
use starcoin_framework::stc_version::Version;
use starcoin_framework::signer;
fun main(account: signer) {
    let version = Version::get(signer::address_of(&account));
    assert!(version == 1, 100);
    let _ = version;
}
}

