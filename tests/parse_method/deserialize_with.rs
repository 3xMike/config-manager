use serde::{Deserialize, Deserializer};

use crate::{assert_ok_and_compare, set_env, test_env};
use config_manager::config;

fn deserialize_with() {
    fn deserialize_i32_and_add_1<'de, D>(de: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        i32::deserialize(de).map(|x| x + 1_i32)
    }

    const I32_KEY: &str = "i32";
    const STRING_KEY: &str = "string";

    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "json", clap(long = "config")),
        __debug_cmd_input__("--config=./tests/data/config.json", "--i32=24", "--string=foobar")
    )]
    struct Init {
        #[source(
            deserialize_with = "deserialize_i32_and_add_1",
            clap(long(init_from = "I32_KEY")),
            env(init_from = "I32_KEY")
        )]
        i32: i32,
        #[source(clap(long(init_from = "STRING_KEY")), env(init_from = "STRING_KEY"))]
        string: String,
    }

    set_env(I32_KEY, 24);
    set_env(STRING_KEY, "foobar");

    assert_ok_and_compare(&Init {
        i32: 25,
        string: "foobar".to_string(),
    });
}

#[test]
fn tests() {
    test_env(vec![deserialize_with]);
}
