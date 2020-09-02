use crate::{gen_client::NetworkRpcClient, GetAccountState, GetStateWithProof};
use anyhow::{anyhow, Result};
use network_api::NetworkService;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateView, StateWithProof};
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::peer_info::PeerId;
use starcoin_types::state_set::ChainStateSet;

#[derive(Clone)]
pub struct RemoteChainStateReader<N>
where
    N: NetworkService,
{
    peer_id: Option<PeerId>,
    state_root: Option<HashValue>,
    client: NetworkRpcClient<N>,
}

impl<N> RemoteChainStateReader<N>
where
    N: NetworkService,
{
    pub fn new(client: NetworkRpcClient<N>) -> Self {
        Self {
            peer_id: None,
            state_root: None,
            client,
        }
    }
    pub fn with(&self, peer_id: PeerId, state_root: HashValue) -> Self {
        Self {
            peer_id: Some(peer_id),
            state_root: Some(state_root),
            client: self.client.clone(),
        }
    }
}

impl<N> ChainStateReader for RemoteChainStateReader<N>
where
    N: NetworkService,
{
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        let peer_id = self
            .peer_id
            .clone()
            .ok_or_else(|| anyhow!("peer id not set"))?;
        let state_root = self
            .state_root
            .clone()
            .ok_or_else(|| anyhow!("state root not set"))?;
        let req = GetStateWithProof {
            state_root,
            access_path: access_path.clone(),
        };
        let client = self.client.clone();
        let state_proof: StateWithProof =
            futures::executor::block_on(client.get_state_with_proof(peer_id, req))?;
        state_proof.proof.verify(
            state_root,
            access_path.clone(),
            state_proof.state.as_deref(),
        )?;
        Ok(state_proof)
    }

    fn get_account_state(&self, account_address: &AccountAddress) -> Result<Option<AccountState>> {
        //TODO: How to verify it
        let peer_id = self
            .peer_id
            .clone()
            .ok_or_else(|| anyhow!("peer id not set"))?;
        let state_root = self
            .state_root
            .clone()
            .ok_or_else(|| anyhow!("state root not set"))?;
        let req = GetAccountState {
            state_root,
            account_address: account_address.to_owned(),
        };
        let client = self.client.clone();
        futures::executor::block_on(async {
            client
                .get_account_state(peer_id, req)
                .await
                .map_err(|e| e.into())
        })
    }

    fn state_root(&self) -> HashValue {
        match self.state_root {
            Some(state_root) => state_root,
            None => unreachable!(),
        }
    }

    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }
}

impl<N> StateView for RemoteChainStateReader<N>
where
    N: NetworkService,
{
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let state_proof = self.get_with_proof(access_path)?;
        Ok(state_proof.state)
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
