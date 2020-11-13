//! account: alice

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: association
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Signer;

    fun transfer_some_token_to_alice_and_bob(signer: &signer) {
        let balance = Account::balance<STC>(Signer::address_of(signer));
        Account::pay_from<STC>(signer, {{alice}}, balance / 4);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    fun propose(signer: &signer) {
        ModifyDaoConfigProposal::propose<STC>(signer, 60 * 60 * 24 * 1000, 0, 50, 0, 0);
    }
}
// check: EXECUTED


//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 250000000


//! new-transaction
//! sender: alice

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun check_state(_signer: &signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 3, (state as u64));
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 250000100


//! new-transaction
//! sender: alice

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun cleanup_proposal_should_fail(_signer: &signer) {
        Dao::destroy_terminated_proposal<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
    }
}
// check: EXECUTED



