use state_api::{ChainStateReader, StateWithProof, StateView};
use types::access_path::AccessPath;
use types::account_address::AccountAddress;
use types::account_state::AccountState;
use types::state_set::ChainStateSet;
use anyhow::Result;
use crypto::HashValue;
use types::peer_info::PeerId;

struct RemoteChainStateReader {
    peer_id: PeerId,
    state_root: HashValue,
}

impl RemoteChainStateReader {
    fn new(peer_id: PeerId, state_root: HashValue) -> Self {
        Self {
            peer_id,
            state_root,
        }
    }
}

impl ChainStateReader for RemoteChainStateReader {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        unimplemented!()
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        unimplemented!()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }
}

impl StateView for RemoteChainStateReader {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
