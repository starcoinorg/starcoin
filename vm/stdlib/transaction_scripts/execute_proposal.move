script {
use 0x1::OnChainConfigDao;
use 0x1::STC;
use 0x1::Signer;

fun execute_proposal<ConfigT: copyable>(account: &signer, proposal_id: u64) {
    OnChainConfigDao::execute<STC::STC, ConfigT>(Signer::address_of(account), proposal_id);
}

spec fun execute_proposal {
    pragma verify = false;
}
}
