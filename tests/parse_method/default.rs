use std::str::FromStr;

use serde::Deserialize;

use crate::{assert_ok_and_compare, test_env};
use config_manager::config;

fn default() {
    #[derive(Debug, PartialEq, Default, Deserialize)]
    struct DotAndNumber {
        inner: i32,
    }

    impl FromStr for DotAndNumber {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.len() < 2 {
                return Err(());
            }
            if let Some('.') = s.chars().next() {
                Ok(DotAndNumber {
                    inner: s[1..].parse().map_err(|_| ())?,
                })
            } else {
                Err(())
            }
        }
    }

    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "json", clap(long = "config")),
        __debug_cmd_input__("--config=./tests/data/config.json")
    )]
    struct Init {
        #[source(default)]
        point: DotAndNumber,
    }

    assert_ok_and_compare(&Init {
        point: Default::default(),
    });
}

fn custom_default() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct DotAndNumber {
        inner: i32,
    }

    impl FromStr for DotAndNumber {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.len() < 2 {
                return Err(());
            }
            if let Some('.') = s.chars().next() {
                Ok(DotAndNumber {
                    inner: s[1..].parse().map_err(|_| ())?,
                })
            } else {
                Err(())
            }
        }
    }

    impl Default for DotAndNumber {
        fn default() -> Self {
            Self { inner: 101 }
        }
    }

    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "json", clap(long = "config")),
        __debug_cmd_input__("--config=./tests/data/config.json")
    )]
    struct Init {
        #[source(default)]
        point: DotAndNumber,
    }

    assert_ok_and_compare(&Init {
        point: Default::default(),
    });
}

#[test]
fn tests() {
    test_env(vec![default, custom_default]);
}
