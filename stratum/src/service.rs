use crate::rpc::{Metadata, StratumRpc, StratumRpcImpl};
use crate::stratum::Stratum;
use anyhow::Result;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_pubsub::Session;
use starcoin_config::NodeConfig;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory};
use std::sync::Arc;

pub struct StratumService {
    config: Arc<NodeConfig>,
    tcp: Option<jsonrpc_tcp_server::Server>,
}

impl ActorService for StratumService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        if let Some(address) = self.config.stratum.get_address() {
            let mut io = MetaIoHandler::default();
            let stratum = ctx.service_ref::<Stratum>()?.clone();
            let rpc = StratumRpcImpl::new(stratum);
            let apis = rpc.to_delegate();
            io.extend_with(apis);
            let server = jsonrpc_tcp_server::ServerBuilder::with_meta_extractor(
                io,
                move |context: &jsonrpc_tcp_server::RequestContext| {
                    Metadata::new(Arc::new(Session::new(context.sender.clone())))
                },
            )
            .start(&address)?;
            self.tcp = Some(server);
        }
        Ok(())
    }
    fn stopped(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        if let Some(tcp) = self.tcp.take() {
            tcp.close()
        }
        Ok(())
    }
}

pub struct StratumServiceFactory;

impl ServiceFactory<StratumService> for StratumServiceFactory {
    fn create(ctx: &mut ServiceContext<StratumService>) -> Result<StratumService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        Ok(StratumService { config, tcp: None })
    }
}
