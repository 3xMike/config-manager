use std::str::FromStr;

use config_manager::config;
use serde::Deserialize;

use crate::{assert_ok_and_compare, test_env};

fn simple_subcommand() {
    #[derive(Debug, PartialEq, Deserialize, clap::Subcommand)]
    enum Subcommand {
        FirstAndOnly(SubcommandArgs),
    }

    #[derive(Debug, PartialEq, Deserialize, clap::Args)]
    struct SubcommandArgs {
        #[clap(long)]
        i32: i32,
        #[clap(long)]
        string: String,
        #[clap(long)]
        opt_i32: Option<i32>,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__(
        "--common=true",
        "first-and-only",
        "--i32=101",
        "--string=foobar",
        "--opt-i32=-1"
    ))]
    struct Simple {
        #[source(clap(long), env, config)]
        common: bool,
        #[subcommand]
        subcommand: Subcommand,
    }

    assert_ok_and_compare(&Simple {
        common: true,
        subcommand: Subcommand::FirstAndOnly(SubcommandArgs {
            i32: 101,
            string: "foobar".to_string(),
            opt_i32: Some(-1),
        }),
    })
}

fn multiple_subcommands() {
    #[derive(Debug, PartialEq, Deserialize, clap::Subcommand)]
    enum Subcommand {
        FooSubcommand(FooSubcommandArgs),
        BarSubcommand(BarSubcommandArgs),
    }

    #[derive(Debug, PartialEq, Deserialize, clap::Args)]
    struct FooSubcommandArgs {}

    #[derive(Debug, PartialEq, Deserialize, clap::Args, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl FromStr for Point {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let msg = "is not a valid point".to_string();
            let mut x_and_y = s
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
                .map(|s| s.split(','))
                .ok_or_else(|| msg.to_string())?;

            let mut fetch_and_parse_coord = || -> Result<i32, Self::Err> {
                x_and_y
                    .next()
                    .ok_or_else(|| msg.to_string())?
                    .parse::<i32>()
                    .map_err(|err| err.to_string())
            };

            let x = fetch_and_parse_coord()?;
            let y = fetch_and_parse_coord()?;
            x_and_y
                .next()
                .is_none()
                .then_some(Point { x, y })
                .ok_or_else(|| msg.to_string())
        }
    }

    #[derive(Debug, PartialEq, Deserialize, clap::Args)]
    struct BarSubcommandArgs {
        #[clap(long)]
        opt_str: Option<String>,
        #[clap(long)]
        bool: bool,
        #[clap(long)]
        point: Point,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__("foo-subcommand"))]
    struct WithFoo {
        #[subcommand]
        subcommand: Subcommand,
    }

    assert_ok_and_compare(&WithFoo {
        subcommand: Subcommand::FooSubcommand(FooSubcommandArgs {}),
    });

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__(
        "bar-subcommand",
        // "--opt-str=null",
        "--point=(1,2)",
    ))]
    struct WithBar {
        #[subcommand]
        subcommand: Subcommand,
    }

    assert_ok_and_compare(&WithBar {
        subcommand: Subcommand::BarSubcommand(BarSubcommandArgs {
            opt_str: None,
            bool: false,
            point: Point { x: 1, y: 2 },
        }),
    });
}

fn optional_subcommand() {
    #[derive(Debug, PartialEq, Deserialize, clap::Subcommand)]
    pub(super) enum SubComm {
        Add {
            #[clap(long)]
            name: Option<String>,
        },
        Sub(SubNested),
    }

    #[derive(Debug, PartialEq, Deserialize, clap::Args)]
    pub(super) struct SubNested {
        #[clap(long)]
        pub(super) field: i32,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__("add", "--name=Mike"))]
    pub(super) struct MainAdd1 {
        #[subcommand]
        pub(super) command: SubComm,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__("add"))]
    pub(super) struct MainAdd2 {
        #[subcommand]
        pub(super) command: SubComm,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__("sub", "--field=2"))]
    pub(super) struct MainSub {
        #[subcommand]
        pub(super) command: SubComm,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__())]
    pub(super) struct MainEmptySub {
        #[subcommand]
        pub(super) command: Option<SubComm>,
    }

    assert_ok_and_compare(&MainAdd1 {
        command: SubComm::Add {
            name: Some("Mike".into()),
        },
    });

    assert_ok_and_compare(&MainAdd2 {
        command: SubComm::Add { name: None },
    });

    assert_ok_and_compare(&MainSub {
        command: SubComm::Sub(SubNested { field: 2 }),
    });

    assert_ok_and_compare(&MainEmptySub { command: None });
}

#[test]
fn subcommand() {
    test_env(vec![
        simple_subcommand,
        multiple_subcommands,
        optional_subcommand,
    ]);
}
