use config_manager::config;

use crate::{assert_ok_and_compare, set_env, test_env};

fn env_and_config_init_from() {
    const ENV_KEY: &str = "myenv";
    const CONFIG_KEY: &str = "int";

    #[derive(Debug, PartialEq)]
    #[config(file(format = "json", env = "config"), __debug_cmd_input__())]
    struct Init {
        #[source(env(init_from = "ENV_KEY"))]
        env: i32,
        #[source(config(init_from = "CONFIG_KEY"))]
        config: i32,
    }

    set_env("config", "./tests/data/config.json");
    set_env("myenv", 1);
    assert_ok_and_compare(&Init { env: 1, config: 1 })
}

fn clap_init_from() {
    const ABOUT: &str = "test clap about";
    const VERSION: &str = "0.0.0";
    const AUTHOR: &str = "me";

    const I32_KEY: &str = "i32";
    const STRING_KEY: &str = "string";

    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "json", clap(long = "config")),
        clap(
            about(init_from = "ABOUT"),
            version(init_from = "VERSION"),
            author(init_from = "AUTHOR")
        ),
        __debug_cmd_input__("--config=./tests/data/config.json", "--i32=24", "--string=foobar")
    )]
    struct Init {
        #[source(clap(long(init_from = "I32_KEY")))]
        i32: i32,
        #[source(clap(long(init_from = "STRING_KEY")))]
        string: String,
    }

    assert_ok_and_compare(&Init {
        i32: 24,
        string: "foobar".to_string(),
    });
}

#[test]
fn tests() {
    test_env(vec![env_and_config_init_from, clap_init_from]);
}
