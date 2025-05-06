use anyhow::Result;
use clap::Parser;
use starcoin_vm2_rpc::{account_api, contract_api, state_api};
use std::fs::{create_dir_all, File};
use std::path::Path;
#[derive(Debug, Parser)]
#[clap(name = "rpc2_generator")]
pub struct RpcSchemaGenerateOpt {
    #[clap(long, short = 'd', default_value = "generated_rpc_schema")]
    /// data dir to generate rpc schema.
    pub data_dir: String,
}

macro_rules! generate_rpc_schema_docs {
    ($data_dir: expr,$($name: ident),+) => {
        ||->Result<()>{
            $(let schema = $name::gen_schema();
              let file_name = format!("{}2.json", stringify!($name));
              let file = File::create($data_dir.join(file_name))?;
              serde_json::to_writer_pretty(file, &schema)?;)*
            Ok(())
        }().unwrap()

    }
}

fn main() {
    let opts = RpcSchemaGenerateOpt::parse();
    generate_rpc_schema_docs!(
        {
            let data_dir = Path::new(&opts.data_dir);
            if !data_dir.exists() {
                create_dir_all(data_dir)?;
            }
            data_dir
        },
        account_api,
        contract_api,
        state_api
    );
}
