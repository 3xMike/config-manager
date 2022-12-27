use std::collections::HashMap;

use config_manager::config;
use serde::Deserialize;

use crate::{assert_ok_and_compare, set_env, test_env};

const CONFIG_NAME: &str = "name";

#[derive(Debug, PartialEq, Deserialize)]
struct Foo {
    data: Bar,
}

#[derive(Debug, PartialEq, Deserialize)]
enum Bar {
    First(Option<i32>),
    Second,
}

#[derive(Debug, PartialEq)]
#[config(
    clap(version, author),
    env_prefix = "a",
    file(format = "json", env = "config", optional = true),
    __debug_cmd_input__("-b=165", "--struct={\"data\": {\"First\": 7}}")
)]
struct MethodConfig {
    #[source(clap, env, config)]
    a: i32,
    #[source(
        env(init_from = "stringify!(env_b)"),
        default = "\"abc\".to_string()",
        config(init_from = "CONFIG_NAME")
    )]
    b: String,
    #[source(config = "bpm", clap(short = 'b'))]
    c: i32,
    #[source(default = "HashMap::new()")]
    d: HashMap<i32, String>,
    #[source(clap(long = "struct"), default = "Foo{data: Bar::Second}")]
    e: Foo,
}

fn input1() {
    set_env("config", "./tests/data/config.json");
    set_env("a_a", 5);

    assert_ok_and_compare(&MethodConfig {
        a: 5,
        b: "Mike".into(),
        c: 165,
        d: HashMap::new(),
        e: Foo {
            data: Bar::First(Some(7)),
        },
    });
}

#[test]
fn big_test() {
    test_env(vec![input1]);
}
