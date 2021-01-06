// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::ChainNetwork;
use starcoin_genesis::{Genesis, GenesisOpt, GENESIS_GENERATED_DIR};
use starcoin_logger::prelude::*;
use starcoin_vm_types::genesis_config::BuiltinNetworkID;
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "genesis_generator")]
pub struct GenesisGeneratorOpt {
    #[structopt(long, short = "n")]
    /// Chain Network to generate genesis, if omit this, generate all network's genesis.
    pub net: Option<BuiltinNetworkID>,
}

fn main() {
    let _logger = starcoin_logger::init();
    let opts = GenesisGeneratorOpt::from_args();
    let networks = match opts.net {
        Some(network) => vec![network],
        None => BuiltinNetworkID::networks(),
    };
    for id in networks {
        // skip test && dev network generate.
        if id.is_test() || id.is_dev() {
            continue;
        }
        let net = ChainNetwork::new_builtin(id);
        let new_genesis =
            Genesis::load_by_opt(GenesisOpt::Fresh, &net).expect("build genesis fail.");
        let generated_genesis = Genesis::load(&net);
        let regenerate = match generated_genesis {
            Ok(generated_genesis) => {
                let regenerate = new_genesis.block().id() != generated_genesis.block().id();
                if regenerate {
                    info!(
                        "Chain net {} previous generated genesis({:?}) not same as new genesis({:?}), overwrite the genesis.",
                        net,
                        generated_genesis.block().id(),
                        new_genesis.block().id()
                    );
                    debug!("old genesis: {}", generated_genesis);
                    debug!("new genesis: {}", new_genesis);
                }
                regenerate
            }
            Err(e) => {
                warn!(
                    "Load generated genesis fail: {:?}, overwrite the genesis.",
                    e
                );
                true
            }
        };
        if regenerate {
            let path = Path::new(GENESIS_GENERATED_DIR).join(net.to_string());
            new_genesis.save(path.as_path()).expect("save genesis fail");
        } else {
            info!(
                "Chain net {} previous generated genesis same as new genesis, do nothing. id: {:?}",
                net,
                new_genesis.block().id()
            );
        }
    }
}
