// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::helper::to_toml;

#[test]
fn test_generate_and_load() -> Result<()> {
    for net in BuiltinNetworkID::networks() {
        let mut opt = StarcoinOpt::default();
        let temp_path = temp_path();
        opt.net = Some(net.into());
        opt.base_data_dir = Some(temp_path.path().to_path_buf());
        let config = NodeConfig::load_with_opt(&opt)?;
        let config2 = NodeConfig::load_with_opt(&opt)?;
        assert_eq!(
            to_toml(&config)?,
            to_toml(&config2)?,
            "test config for network {} fail.",
            net
        );
    }
    Ok(())
}

#[test]
fn test_custom_chain_genesis() -> Result<()> {
    let net = ChainNetworkID::from_str("test1:123")?;
    let temp_path = temp_path();
    let opt = StarcoinOpt {
        net: Some(net),
        base_data_dir: Some(temp_path.path().to_path_buf()),
        genesis_config: Some(BuiltinNetworkID::Test.to_string()),
        ..StarcoinOpt::default()
    };
    let config = NodeConfig::load_with_opt(&opt)?;
    let config2 = NodeConfig::load_with_opt(&opt)?;
    assert_eq!(
        config, config2,
        "test config for network {:?} fail.",
        opt.net
    );
    Ok(())
}

#[test]
fn test_genesis_config_save_and_load() -> Result<()> {
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    genesis_config.consensus_config.base_block_time_target = 10000000;
    let temp_path = temp_path();
    let file_path = temp_path.path().join(GENESIS_CONFIG_FILE_NAME);
    genesis_config.save(file_path.as_path())?;
    let genesis_config2 = GenesisConfig::load(file_path.as_path())?;
    assert_eq!(genesis_config, genesis_config2);
    Ok(())
}

#[test]
fn test_example_config_compact() -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let example_dir = path.join("example");
    for net in BuiltinNetworkID::networks() {
        if net.is_dev() || net.is_test() || !net.genesis_config().is_ready() {
            continue;
        }
        let net_str = net.to_string();
        let args = vec![
            "starcoin",
            "-n",
            net_str.as_str(),
            "-d",
            example_dir.to_str().unwrap(),
            //Network
            "--seed",
            "/ip4/1.2.3.3/tcp/9840/p2p/QmRZ6ZwVzhJ6xpVV1CEve2RKiUzK4y2pSx3eg2cvQMsT4f",
            "--seed",
            "/ip4/1.2.3.4/tcp/9840/p2p/12D3KooWCfUex27aoqaKScponiLB4N4FWbgmbHYjVoRebGrQaRYk",
            "--node-name",
            "alice-node1",
            //P2P
            "--p2prpc-default-global-api-quota",
            "2000/s",
            //HTTP
            "--http-apis",
            "safe",
            "--jsonrpc-default-global-api-quota",
            "2000/s",
            "--jsonrpc-custom-user-api-quota",
            "chain.info=100/s",
            //TCP
            "--tcp-apis",
            "safe",
            //Websocket
            "--websocket-apis",
            "pubsub",
            //IPC
            "--ipc-apis",
            "ipc",
            //Miner
            "--miner-thread",
            "3",
            //TXPool
            "--txpool-max-count",
            "8192",
            //Logger
            "--logger-max-backup",
            "100",
            //Metrics
            "--metrics-address",
            "127.0.0.1",
            //Storage
            "--rocksdb-max-open-files",
            "40960",
            //Vault
            "--vault-dir",
            "/data/my_starcoin_vault",
        ];
        let opt = StarcoinOpt::from_iter_safe(args)?;

        let config = NodeConfig::load_with_opt(&opt)?;
        let config2 = NodeConfig::load_with_opt(&opt)?;
        assert_eq!(
            to_toml(&config)?,
            to_toml(&config2)?,
            "test config for network {} fail.",
            net
        );
    }
    Ok(())
}
