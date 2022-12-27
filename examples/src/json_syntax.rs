use std::collections::HashMap;

use config_manager::{config, ConfigInit};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
enum MyResult {
    Ok(bool),
    Error,
}

#[derive(Debug, Deserialize)]
struct ApprovedInt {
    value: i32,
    is_approved: bool,
}

#[derive(Debug)]
#[config]
struct Config {
    #[source(clap)]
    int: i32,
    #[source(clap)]
    string: String,
    #[source(clap)]
    array: Vec<MyResult>,
    #[source(clap)]
    map_of_optionals: HashMap<i32, Option<i32>>,
    #[source(clap)]
    class: ApprovedInt,
}

fn main() {
    let cfg = <Config as ConfigInit>::parse();
    dbg!(cfg);
}
