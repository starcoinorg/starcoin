script {
use 0x1::Signer;
use 0x1::PackageTxnManager;
use 0x1::CoreAddresses;

fun genesis_init(genesis_account: &signer) {
    assert(Signer::address_of(genesis_account) == 0x1,1);
    PackageTxnManager::grant_maintainer(genesis_account, CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
}
}