use starcoin_config::NodeConfig;

use clap::Clap;
#[derive(Clap)]
#[clap(version = "1.0", author = "starcoin")]
struct Opts {
    #[clap(short = "o", long = "output_dir", default_value = "starcoin")]
    output_file: String,
}

fn main() {
    let opts: Opts = Opts::parse();
    NodeConfig::load(&opts.output_file).expect("write file should ok");
    println!("generated config.toml in dir {}", opts.output_file);
}
