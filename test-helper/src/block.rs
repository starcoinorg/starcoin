use starcoin_chain::BlockChain;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_types::account::Account;
use starcoin_types::block::Block;
use starcoin_vm_types::transaction::SignedUserTransaction;

pub fn create_new_block(
    chain: &BlockChain,
    account: &Account,
    txns: Vec<SignedUserTransaction>,
) -> anyhow::Result<Block> {
    let (template, _) = chain.create_block_template(
        *account.address(),
        None,
        txns,
        vec![],
        None,
        vec![],
        HashValue::zero(),
    )?;
    chain
        .consensus()
        .create_block(template, chain.time_service().as_ref())
}
