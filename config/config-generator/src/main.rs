use starcoin_config::{
    NodeConfig,save_config,
};
fn main() {
    let config_template = NodeConfig::default();
    save_config(&config_template, "config.template.toml").expect("write file should ok");
}
