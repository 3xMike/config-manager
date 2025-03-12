#![allow(dead_code)]

use std::collections::HashMap;

use config_manager::{config, ConfigInit};

const SUFFIX: &str = "_env";

/// Long about for my App.
#[derive(Debug)]
#[config(
    clap(version, author, long_about),
    env_prefix = "demo",
    file(
        format = "toml",
        clap(long = "config", short = 'c', help = "path to configuration file"),
        env = "demo_config",
        default = "./config.toml"
    )
)]
struct MethodConfig {
    /// This docs will be help and long_help.
    #[source(clap(long, short, help, long_help, help_heading = "A heading"))]
    a: i32,
    #[source(env(init_from = &format!("b{SUFFIX}")), default = "abc")]
    b: String,
    #[source(config = "bpm")]
    c: i32,
    #[source(default = HashMap::new())]
    d: HashMap<i32, String>,
    #[source(clap(flag), default = false)]
    f: bool,
}

fn main() {
    dbg!(MethodConfig::parse().unwrap());
}
