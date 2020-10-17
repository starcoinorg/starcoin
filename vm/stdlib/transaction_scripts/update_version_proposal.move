script {
use 0x1::Version;
use 0x1::OnChainConfigDao;
use 0x1::STC;

fun update_version(account: &signer,
    major: u64,
    exec_delay: u64) {
    let version = Version::new_version(major);
    OnChainConfigDao::propose_update<STC::STC, Version::Version>(account, version, exec_delay);
}

spec fun update_version {
    pragma verify = false;
}
}
