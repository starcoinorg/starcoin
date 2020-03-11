use starcoin_config::{
    NodeConfig,save_config,
};

use clap::Clap;
#[derive(Clap)]
#[clap(version = "1.0", author = "starcoin")]
struct Opts {
    #[clap(short = "o", long = "output", default_value = "config.template.toml")]
    output_file: String,
}

fn main() {
    let opts: Opts = Opts::parse();
    let config_template = NodeConfig::default();
    save_config(&config_template, opts.output_file).expect("write file should ok");
}
