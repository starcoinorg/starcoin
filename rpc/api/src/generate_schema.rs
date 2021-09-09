use anyhow::Result;
use starcoin_rpc_api::{
    account, chain, contract_api, debug, miner, network_manager, node, node_manager, state,
    sync_manager, txpool,
};
use std::fs::{create_dir_all, File};
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "genesis_generator")]
pub struct RpcSchemaGenerateOpt {
    #[structopt(long, short = "d", default_value = "generated_rpc_schema")]
    /// data dir to generate rpc schema.
    pub data_dir: String,
}

macro_rules! generate_rpc_schema_docs {
    ($data_dir: expr,$($name: ident),+) => {
        ||->Result<()>{
            $(let schema = $name::gen_client::Client::gen_schema();
              let file_name = format!("{}.json", stringify!($name));
              let file = File::create($data_dir.join(file_name))?;
              serde_json::to_writer_pretty(file, &schema)?;)*
            Ok(())
        }().unwrap()

    }
}

fn main() {
    let opts = RpcSchemaGenerateOpt::from_args();
    generate_rpc_schema_docs!(
        {
            let data_dir = Path::new(&opts.data_dir);
            if !data_dir.exists() {
                create_dir_all(data_dir)?;
            }
            data_dir
        },
        account,
        chain,
        contract_api,
        debug,
        miner,
        network_manager,
        node,
        node_manager,
        state,
        sync_manager,
        txpool
    );
}
