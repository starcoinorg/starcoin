use starcoin_config::{NodeConfig, StarcoinOpt};
use structopt::StructOpt;

fn main() {
    let opts: StarcoinOpt = StarcoinOpt::from_args();
    if opts.data_dir.is_none() {
        println!("please set data_dir to generate config.");
        return;
    }
    let config = NodeConfig::load_with_opt(&opts).expect("write file should ok");
    println!("generated config.toml in dir {:?}", config.base.data_dir());
}
