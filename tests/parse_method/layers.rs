use crate::*;
use config_manager::config;

#[test]
fn sources_absense() {
    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "toml", default = "./tests/data/config.toml"),
        env_prefix = "my",
        __debug_cmd_input__()
    )]
    struct Cfg {
        #[source(default)]
        int: i32,
    }
    set_env("my_int", "1");

    assert_ok_and_compare(&Cfg { int: 0 })
}

#[test]
fn sources_priority() {
    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "toml", default = "./tests/data/config.toml"),
        env_prefix = "my",
        __debug_cmd_input__("--clap=2", "--config=10")
    )]
    struct Order {
        #[source(clap, env, config)]
        env: i32,
        #[source(env, config, clap)]
        clap: i32,
        #[source(config, clap)]
        config: i32,
    }
    set_env("my_env", "1");

    assert_ok_and_compare(&Order {
        env: 1,
        clap: 2,
        config: 3,
    })
}

#[test]
fn fallback_sources() {
    // No clap, because there's no reason to create a fallback var in cmd
    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "toml", default = "./tests/data/config.toml"),
        env_prefix = "my",
        __debug_cmd_input__()
    )]
    struct Fallback {
        #[source(env, env = "env2", default)]
        env: i32,
        #[source(config = "lan", default, config)]
        config: i32,
    }
    set_env("env2", "2");

    assert_ok_and_compare(&Fallback { env: 2, config: 3 })
}

#[test]
fn short_sources() {
    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "toml", default = "./tests/data/config.toml"),
        env_prefix = "my",
        __debug_cmd_input__("--clap=2")
    )]
    struct Cfg {
        #[source(default)]
        default: i32,
        #[source(env)]
        env: i32,
        #[source(clap)]
        clap: i32,
        #[source(config)]
        config: i32,
    }
    set_env("my_env", "1");

    assert_ok_and_compare(&Cfg {
        default: 0,
        env: 1,
        clap: 2,
        config: 3,
    })
}
