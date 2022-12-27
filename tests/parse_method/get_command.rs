use clap::arg;
use config_manager::__private::clap;
use config_manager::{config, ConfigInit};
use std::collections::HashMap;

#[test]
fn get_command() {
    #[allow(dead_code)]
    #[derive(Debug)]
    #[config(clap(version, author))]
    struct Config {
        #[source(clap(long, short))]
        a: i32,
        #[source(env, default = "\"abc\".to_string()")]
        b: String,
        #[source(clap, config = "bpm")]
        c: i32,
        #[source(default)]
        d: HashMap<i32, String>,
        #[source(clap(long = "field", short = 'q', help = "some field"))]
        e: i32,
    }
    let a = arg!(-a --a <a>);
    let c = arg!(--c <c> );
    let e = arg!(-q --field <field> "some field");

    let command = Config::get_command();

    assert_eq!(command.get_about(), None);
    assert_eq!(command.get_author(), Some(clap::crate_authors!()));
    assert_eq!(command.get_version(), Some(clap::crate_version!()));
    assert_eq!(command.get_arguments().collect::<Vec<_>>(), &[&a, &c, &e]);
}
