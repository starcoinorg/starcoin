// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use bcs_ext::BCSCodec;
use scmd::{CommandAction, ExecContext};
use starcoin_network_rpc_api::Ping;
use starcoin_rpc_api::types::StrView;
use starcoin_types::peer_info::PeerId;
use structopt::StructOpt;

/// Call peer method by p2p network, just for diagnose network problem.
#[derive(Debug, StructOpt)]
#[structopt(name = "call_peer")]
pub struct CallPeerOpt {
    #[structopt(short = "p", long = "peer-id")]
    peer_id: PeerId,
    /// rpc path, if absent, use ping method.
    #[structopt(short = "r", long = "rpc-method")]
    rpc_method: Option<String>,
    /// request message serialize by lcs and encode by hex
    #[structopt(short = "m", long = "message")]
    message: Option<String>,
}

pub struct CallPeerCommand;

impl CommandAction for CallPeerCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CallPeerOpt;
    type ReturnItem = StrView<Vec<u8>>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let (rpc_method, message) = match (opt.rpc_method.as_ref(), opt.message.as_ref()) {
            (None, _) => {
                let ping_msg = Ping {
                    msg: "ping_by_cmd".to_string(),
                    err: false,
                };
                ("ping".to_string(), ping_msg.encode()?)
            }
            (Some(rpc_method), Some(message)) => (rpc_method.clone(), hex::decode(message)?),
            (Some(_rpc_method), None) => {
                bail!("Please input call message.")
            }
        };
        client.network_call_peer(opt.peer_id.to_string(), rpc_method, StrView(message))
    }
}
