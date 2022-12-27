use std::collections::HashMap;

use config_manager::{config, ConfigInit};

const SUFFIX: &str = "_env";

#[derive(Debug)]
#[config(
    clap(version, author),
    env_prefix = "demo",
    file(
        format = "toml",
        clap(long = "config", short = 'c', help = "path to configuration file"),
        env = "demo_config",
        default = "./config.toml"
    )
)]
struct MethodConfig {
    #[source(clap(long, short))]
    a: i32,
    #[source(
        env(init_from = "&format!(\"b{}\", SUFFIX)"),
        default = "\"abc\".to_string()"
    )]
    b: String,
    #[source(config = "bpm")]
    c: i32,
    #[source(default = "HashMap::new()")]
    d: HashMap<i32, String>,
}

fn main() {
    dbg!(MethodConfig::parse().unwrap());
}
