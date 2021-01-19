// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::helper::to_toml;
use network_p2p_types::MultiaddrWithPeerId;

#[test]
fn test_generate_and_load() -> Result<()> {
    for net in BuiltinNetworkID::networks() {
        let mut opt = StarcoinOpt::default();
        let temp_path = temp_path();
        opt.net = Some(net.into());
        opt.data_dir = Some(temp_path.path().to_path_buf());
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
        data_dir: Some(temp_path.path().to_path_buf()),
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
    genesis_config.timestamp = 1000;
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
        let mut opt = StarcoinOpt {
            net: Some(net.into()),
            data_dir: Some(example_dir.clone()),
            ..StarcoinOpt::default()
        };
        opt.network.seeds = Some(vec![MultiaddrWithPeerId::from_str(
            "/ip4/198.51.100.19/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV",
        )?]);

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
