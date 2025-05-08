
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::time::Duration;
use starcoin_account_api::AccountProvider;
use starcoin_config::ChainNetworkID;
use starcoin_node::NodeHandle;

pub trait CliStateTrait {
    fn new(
        net: ChainNetworkID,
        client: Arc<RpcClient>,
        watch_timeout: Option<Duration>,
        node_handle: Option<NodeHandle>,
        account_client: Box<dyn AccountProvider>,
    ) -> Self;
}
