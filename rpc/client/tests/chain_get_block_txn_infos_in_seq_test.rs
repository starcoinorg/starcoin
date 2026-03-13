use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_crypto::{keygen::KeyGen, HashValue};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use starcoin_transaction_builder::{build_transfer_from_association, DEFAULT_EXPIRATION_TIME};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    account_address, multi_transaction::MultiSignedUserTransaction,
    transaction::SignedUserTransaction,
};
use starcoin_vm2_types::transaction::Transaction as Transaction2;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[stest::test(timeout = 180)]
fn test_chain_get_block_txn_infos_in_seq() -> Result<()> {
    let mut node_config = NodeConfig::random_for_test();
    node_config.miner.disable_miner_client = Some(false);
    let config = Arc::new(node_config);

    let ipc_file = config.rpc.get_ipc_file();

    let node_handle = test_helper::run_node_by_config(config.clone())?;
    std::thread::sleep(Duration::from_millis(500));

    let txpool = node_handle.txpool();

    let vm1_txn1 = gen_vm1_user_txn(&config, 0);
    let vm1_txn2 = gen_vm1_user_txn(&config, 1);

    let import_result = txpool.add_txns(vec![vm1_txn1.clone(), vm1_txn2.clone()])?;
    assert!(import_result[0].is_ok());
    assert!(import_result[1].is_ok());

    let vm2_txn1 = gen_vm2_user_txn(&config, 0);
    let vm2_txn2 = gen_vm2_user_txn(&config, 1);

    let multi_txns = vec![
        MultiSignedUserTransaction::try_from(vm2_txn1.clone())?,
        MultiSignedUserTransaction::try_from(vm2_txn2.clone())?,
    ];
    let import_result2 =
        txpool.add_txns_multi_signed(multi_txns, false, Some("test-client".to_string()))?;
    assert!(import_result2[0].is_ok());
    assert!(import_result2[1].is_ok());

    std::thread::sleep(Duration::from_millis(500));

    let block = node_handle.generate_block()?;
    let block_hash = block.id();

    std::thread::sleep(Duration::from_millis(500));

    let client = RpcClient::connect_ipc(ipc_file)?;

    let (txn_infos_in_seq, vm1_infos, vm2_infos) =
        wait_for_consistent_txn_infos(&client, block_hash, Duration::from_secs(60))?;

    assert!(
        !txn_infos_in_seq.is_empty(),
        "Should have at least block meta transaction"
    );

    for (i, info) in txn_infos_in_seq.iter().enumerate() {
        match info {
            starcoin_rpc_api::types::TransactionInfoViewEnum::VM1(vm1_info) => {
                info!(
                    "Index {}: VM1 transaction, index={}, hash={:?}",
                    i, vm1_info.transaction_index, vm1_info.transaction_hash
                );
            }
            starcoin_rpc_api::types::TransactionInfoViewEnum::VM2(vm2_info) => {
                info!(
                    "Index {}: VM2 transaction, index={}, hash={:?}",
                    i, vm2_info.transaction_index, vm2_info.transaction_hash
                );
            }
        }
    }

    if let starcoin_rpc_api::types::TransactionInfoViewEnum::VM2(first_txn) = &txn_infos_in_seq[0] {
        assert_eq!(
            first_txn.transaction_index, 0,
            "First transaction should have index 0"
        );
    } else {
        panic!("First transaction should be VM2 block meta transaction");
    }

    for (i, info) in txn_infos_in_seq.iter().enumerate() {
        let txn_index = match info {
            starcoin_rpc_api::types::TransactionInfoViewEnum::VM1(vm1_info) => {
                vm1_info.transaction_index
            }
            starcoin_rpc_api::types::TransactionInfoViewEnum::VM2(vm2_info) => {
                vm2_info.transaction_index
            }
        };
        assert_eq!(
            txn_index as usize, i,
            "Transaction index should match position in array"
        );
    }

    let vm2_count = txn_infos_in_seq
        .iter()
        .filter(|info| {
            matches!(
                info,
                starcoin_rpc_api::types::TransactionInfoViewEnum::VM2(_)
            )
        })
        .count();

    assert!(
        vm2_count >= 1,
        "Should have at least 1 VM2 transaction (block meta)"
    );

    assert_eq!(
        txn_infos_in_seq.len(),
        vm1_infos.len() + vm2_infos.len(),
        "Total transaction count should match sum of VM1 and VM2"
    );

    if !vm1_infos.is_empty() {
        let vm1_hashes: std::collections::HashSet<_> =
            vm1_infos.iter().map(|info| info.transaction_hash).collect();

        let seq_vm1_hashes: std::collections::HashSet<_> = txn_infos_in_seq
            .iter()
            .filter_map(|info| {
                if let starcoin_rpc_api::types::TransactionInfoViewEnum::VM1(vm1_info) = info {
                    Some(vm1_info.transaction_hash)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            vm1_hashes, seq_vm1_hashes,
            "VM1 transaction hashes should match"
        );
    }

    if !vm2_infos.is_empty() {
        let vm2_hashes: std::collections::HashSet<_> =
            vm2_infos.iter().map(|info| info.transaction_hash).collect();

        let seq_vm2_hashes: std::collections::HashSet<_> = txn_infos_in_seq
            .iter()
            .filter_map(|info| {
                if let starcoin_rpc_api::types::TransactionInfoViewEnum::VM2(vm2_info) = info {
                    Some(vm2_info.transaction_hash)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            vm2_hashes, seq_vm2_hashes,
            "VM2 transaction hashes should match"
        );
    }

    client.close();
    node_handle.stop()?;

    Ok(())
}

fn wait_for_consistent_txn_infos(
    client: &RpcClient,
    block_hash: HashValue,
    timeout: Duration,
) -> Result<(
    Vec<starcoin_rpc_api::types::TransactionInfoViewEnum>,
    Vec<starcoin_rpc_api::types::TransactionInfoView>,
    Vec<starcoin_vm2_types::view::TransactionInfoView>,
)> {
    let deadline = Instant::now() + timeout;
    let mut last_err: Option<anyhow::Error>;
    loop {
        let seq_infos = client.chain_get_block_txn_infos_in_seq(block_hash);
        let vm1_infos = client.chain_get_block_txn_infos(block_hash);
        let vm2_infos = client.chain_get_block_txn_infos2(block_hash);

        match (seq_infos, vm1_infos, vm2_infos) {
            (Ok(seq_infos), Ok(vm1_infos), Ok(vm2_infos)) => {
                let has_gap = seq_infos.iter().enumerate().any(|(i, info)| {
                    let index = match info {
                        starcoin_rpc_api::types::TransactionInfoViewEnum::VM1(vm1_info) => {
                            vm1_info.transaction_index
                        }
                        starcoin_rpc_api::types::TransactionInfoViewEnum::VM2(vm2_info) => {
                            vm2_info.transaction_index
                        }
                    };
                    index as usize != i
                });

                let vm1_hashes: std::collections::HashSet<_> =
                    vm1_infos.iter().map(|info| info.transaction_hash).collect();
                let seq_vm1_hashes: std::collections::HashSet<_> = seq_infos
                    .iter()
                    .filter_map(|info| {
                        if let starcoin_rpc_api::types::TransactionInfoViewEnum::VM1(vm1_info) =
                            info
                        {
                            Some(vm1_info.transaction_hash)
                        } else {
                            None
                        }
                    })
                    .collect();

                let vm2_hashes: std::collections::HashSet<_> =
                    vm2_infos.iter().map(|info| info.transaction_hash).collect();
                let seq_vm2_hashes: std::collections::HashSet<_> = seq_infos
                    .iter()
                    .filter_map(|info| {
                        if let starcoin_rpc_api::types::TransactionInfoViewEnum::VM2(vm2_info) =
                            info
                        {
                            Some(vm2_info.transaction_hash)
                        } else {
                            None
                        }
                    })
                    .collect();

                if !seq_infos.is_empty()
                    && !has_gap
                    && seq_infos.len() == vm1_infos.len() + vm2_infos.len()
                    && vm1_hashes == seq_vm1_hashes
                    && vm2_hashes == seq_vm2_hashes
                {
                    return Ok((seq_infos, vm1_infos, vm2_infos));
                }

                let vm1_only_in_seq: Vec<_> = seq_vm1_hashes
                    .difference(&vm1_hashes)
                    .take(3)
                    .cloned()
                    .collect();
                let vm1_missing_in_seq: Vec<_> = vm1_hashes
                    .difference(&seq_vm1_hashes)
                    .take(3)
                    .cloned()
                    .collect();
                let vm2_only_in_seq: Vec<_> = seq_vm2_hashes
                    .difference(&vm2_hashes)
                    .take(3)
                    .cloned()
                    .collect();
                let vm2_missing_in_seq: Vec<_> = vm2_hashes
                    .difference(&seq_vm2_hashes)
                    .take(3)
                    .cloned()
                    .collect();

                last_err = Some(anyhow::format_err!(
                    "waiting block {} inconsistent results: has_gap={}, seq_len={}, vm1_len={}, vm2_len={}, vm1_only_in_seq={:?}, vm1_missing_in_seq={:?}, vm2_only_in_seq={:?}, vm2_missing_in_seq={:?}",
                    block_hash,
                    has_gap,
                    seq_infos.len(),
                    vm1_infos.len(),
                    vm2_infos.len(),
                    vm1_only_in_seq,
                    vm1_missing_in_seq,
                    vm2_only_in_seq,
                    vm2_missing_in_seq
                ));
            }
            (seq_res, vm1_res, vm2_res) => {
                let seq_err = seq_res.err().map(|e| format!("seq err: {e:?}"));
                let vm1_err = vm1_res.err().map(|e| format!("vm1 err: {e:?}"));
                let vm2_err = vm2_res.err().map(|e| format!("vm2 err: {e:?}"));
                last_err = Some(anyhow::format_err!(
                    "waiting block {} failed: {:?} {:?} {:?}",
                    block_hash,
                    seq_err,
                    vm1_err,
                    vm2_err
                ));
            }
        }

        if Instant::now() >= deadline {
            return Err(last_err.unwrap_or_else(|| {
                anyhow::format_err!(
                    "timeout waiting consistent txn infos in seq for block {}",
                    block_hash
                )
            }));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn gen_vm1_user_txn(config: &NodeConfig, seq_number: u64) -> SignedUserTransaction {
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let txn = build_transfer_from_association(
        account_address,
        seq_number,
        10000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net(),
    );
    txn.as_signed_user_txn().unwrap().clone()
}

fn gen_vm2_user_txn(config: &NodeConfig, seq_number: u64) -> Transaction2 {
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = starcoin_vm2_types::account_address::AccountAddress::from_bytes(
        account_address::from_public_key(&public_key).to_vec(),
    )
    .unwrap();

    starcoin_vm2_test_helper::txn::build_transfer_from_association(
        account_address,
        seq_number,
        10000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net(),
    )
}
