use std::collections::{HashMap, HashSet};

use config_manager::*;
use serde::Deserialize;

#[test]
fn test_env_prefix() {
    #[config(env_prefix = "my")]
    struct Env {
        #[source(env)]
        field: i32,
        #[source(env = "find_me")]
        int: i32,
        #[flatten]
        flat: Flat,
    }

    #[derive(Deserialize, Flatten)]
    struct Flat {
        #[source(env)]
        f: i32,
    }

    let res = Env::parse_options(HashSet::from([
        ConfigOption::EnvPrefix("your".to_string()),
        ConfigOption::ExplicitSource(Source::Env(HashMap::from([
            ("your_field".to_string(), "0".to_string()),
            ("find_me".to_string(), "1".to_string()),
            ("your_f".to_string(), "2".to_string()),
        ]))),
        ConfigOption::ExplicitSource(Source::Clap(ClapSource::None)),
    ]))
    .unwrap();

    assert_eq!(res.field, 0);
    assert_eq!(res.int, 1);
    assert_eq!(res.flat.f, 2);
}

#[test]
fn test_config_source() {
    #[config(file(format = "yaml", default = "not found"))]
    struct Config {
        #[source(config)]
        int: i32,
        #[source(config = "input.int")]
        field: i32,
    }

    let res = Config::parse_options(HashSet::from([
        ConfigOption::ExplicitSource(Source::ConfigFiles(vec![FileOptions {
            format: config::FileFormat::Toml,
            path: "tests/data/config.toml".to_string(),
        }])),
        ConfigOption::ExplicitSource(Source::Clap(ClapSource::None)),
    ]))
    .unwrap();

    assert_eq!(res.field, 5);
    assert_eq!(res.int, 1);
}

#[test]
fn test_clap_source() {
    use clap;

    #[config]
    struct Clap1 {
        #[source(clap, default = 0)]
        field: i32,
    }

    #[config(__debug_cmd_input__("Options have priority"))]
    struct Clap2 {
        #[source(clap, default = 0)]
        field: i32,
    }

    let opt1 = HashSet::from([ConfigOption::ExplicitSource(Source::Clap(ClapSource::None))]);
    let opt2 = HashSet::from([ConfigOption::ExplicitSource(Source::Clap(
        ClapSource::Args(vec!["--field=1".to_string()]),
    ))]);
    let matches = clap::Command::new("")
        .arg(
            clap::Arg::new("field")
                .long("my_f")
                .required(true)
                .num_args(1),
        )
        .get_matches_from(vec!["", "--my_f=2"]);
    let opt3 = HashSet::from([ConfigOption::ExplicitSource(Source::Clap(
        ClapSource::Matches(matches),
    ))]);

    let res11 = Clap1::parse_options(opt1.clone()).unwrap();
    let res12 = Clap1::parse_options(opt2.clone()).unwrap();
    let res13 = Clap1::parse_options(opt3.clone()).unwrap();
    let res21 = Clap2::parse_options(opt1).unwrap();
    let res22 = Clap2::parse_options(opt2).unwrap();
    let res23 = Clap2::parse_options(opt3).unwrap();

    assert_eq!(res11.field, 0);
    assert_eq!(res12.field, 1);
    assert_eq!(res13.field, 2);
    assert_eq!(res21.field, 0);
    assert_eq!(res22.field, 1);
    assert_eq!(res23.field, 2);
}

#[test]
fn test_env_source() {
    #[config(env_prefix = "my")]
    struct Env {
        #[source(env)]
        field: i32,
        #[source(env = "find_me")]
        int: i32,
    }

    let res = Env::parse_options(HashSet::from([
        ConfigOption::ExplicitSource(Source::Env(HashMap::from([
            ("my_field".to_string(), "0".to_string()),
            ("find_me".to_string(), "1".to_string()),
        ]))),
        ConfigOption::ExplicitSource(Source::Clap(ClapSource::None)),
    ]))
    .unwrap();

    assert_eq!(res.field, 0);
    assert_eq!(res.int, 1);
}
