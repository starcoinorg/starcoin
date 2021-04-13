use crate::account_config::stc_type_tag;
use crate::language_storage::TypeTag;
use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct DaoConfig {
    /// after proposal created, how long use should wait before he can vote.
    pub voting_delay: u64,
    /// how long the voting window is.
    pub voting_period: u64,
    /// the quorum rate to agree on the proposal.
    /// if 50% votes needed, then the voting_quorum_rate should be 50.
    /// it should between (0, 100].
    pub voting_quorum_rate: u8,
    /// how long the proposal should wait before it can be executed.
    pub min_action_delay: u64,
}

impl OnChainConfig for DaoConfig {
    const MODULE_IDENTIFIER: &'static str = "Dao";
    const CONF_IDENTIFIER: &'static str = "DaoConfig";

    fn type_params() -> Vec<TypeTag> {
        let params = vec![stc_type_tag()];
        params
    }
}
