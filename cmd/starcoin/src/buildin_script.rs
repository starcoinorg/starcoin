use anyhow::bail;
use starcoin_vm_runtime::transaction_scripts::{
    ACCEPT_COIN_TXN, CREATE_ACCOUNT_TXN, MINT_TXN, PEER_TO_PEER_TXN,
};
use std::str::FromStr;

/// TODO: once transaction builder is stable in our codebase,
/// replace the `BuildinScript` with that.
#[derive(Debug, Clone)]
pub enum BuildinScript {
    PeerToPeer,
    AcceptCoin,
    CreateAccount,
    Mint,
}

impl BuildinScript {
    pub fn script_code(&self) -> Vec<u8> {
        match self {
            BuildinScript::PeerToPeer => PEER_TO_PEER_TXN.clone(),
            BuildinScript::AcceptCoin => ACCEPT_COIN_TXN.clone(),
            BuildinScript::CreateAccount => CREATE_ACCOUNT_TXN.clone(),
            BuildinScript::Mint => MINT_TXN.clone(),
        }
    }
}

impl FromStr for BuildinScript {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let script = match s {
            "peer_to_peer" => BuildinScript::PeerToPeer,
            "accept_coin" => BuildinScript::AcceptCoin,
            "create_account" => BuildinScript::CreateAccount,
            "mint" => BuildinScript::Mint,
            _ => bail!("unknown script name"),
        };
        Ok(script)
    }
}
