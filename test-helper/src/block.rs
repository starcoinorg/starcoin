// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_chain::{verifier::FullVerifier, BlockChain, ChainReader};
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_types::{
    account_address::AccountAddress, block::ExecutedBlock,
    multi_transaction::MultiSignedUserTransaction,
};
use starcoin_vm_types::transaction::Transaction;

pub fn create_block_with_transactions(
    chain: &mut BlockChain,
    net: &ChainNetwork,
    miner: AccountAddress,
    transactions: Vec<Transaction>,
) -> anyhow::Result<(ExecutedBlock, HashValue)> {
    let input_txn_len = transactions.len();
    let multi_txns: Vec<MultiSignedUserTransaction> = transactions
        .into_iter()
        .map(|txn| MultiSignedUserTransaction::from(txn.as_signed_user_txn().unwrap().clone()))
        .collect();
    let (block_template, _) = chain.create_block_template_simple_with_txns(miner, multi_txns)?;
    let block = chain
        .consensus()
        .create_block(block_template, net.time_service().as_ref())?;
    let executed_block = chain.apply_with_verifier::<FullVerifier>(block.clone())?;

    // Check all txn valid
    assert_eq!(executed_block.block().transactions().len(), input_txn_len);

    Ok((executed_block, chain.chain_state_reader().state_root()))
}
