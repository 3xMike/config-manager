use clap::arg;
use config_manager::__private::clap::{builder::styling, ColorChoice, ValueHint};
use config_manager::{config, ConfigInit};
use std::collections::HashMap;

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

#[test]
fn get_command() {
    #[allow(dead_code)]
    #[config(clap(version, author))]
    struct Config {
        #[source(clap(long, short))]
        a: i32,
        #[source(env, default = "abc")]
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
    assert_eq!(command.get_author(), Some(clap::crate_authors!("\n")));
    assert_eq!(command.get_version(), Some(clap::crate_version!()));
    assert_eq!(command.get_arguments().collect::<Vec<_>>(), &[&a, &c, &e]);
}

#[test]
fn all_clap_attrs() {
    #[config(clap(
        name = "a",
        version = "b",
        author = "c",
        about = "d",
        long_about = "e",
        color = ColorChoice::Never,
        styles = STYLES,
        term_width = 5,
        max_term_width = 10,
        disable_version_flag,
        next_line_help,
        disable_help_flag,
        disable_colored_help,
        help_expected,
        hide_possible_values,
        bin_name = "f",
        display_name = "g",
        after_help = "h",
        after_long_help = "i",
        before_help = "j",
        before_long_help = "k",
        long_version = "l",
        override_usage = "m",
        override_help = "n",
        help_template = "o",
        next_help_heading = "p",
        next_display_order = 5,
        allow_missing_positional,
        arg_required_else_help
    ))]
    struct Config {
        #[source(clap(
            help = "a",
            long_help = "b",
            short = 'c',
            long = "d",
            flag,
            help_heading = "e",
            alias = "f",
            short_alias = 'g',
            aliases = ["h1", "h2", "h3"],
            short_aliases = ['i', 'j', 'k'],
            visible_alias = "l",
            visible_short_alias = 'm',
            visible_aliases = ["n1", "n2", "n3"],
            visible_short_aliases = ['o', 'p', 'q'],
            index = 5,
            last,
            requires = "other_field",
            exclusive,
            value_name = "r",
            value_hint = ValueHint::Username,
            ignore_case,
            allow_hyphen_values,
            allow_negative_numbers,
            require_equals,
            display_order = 10,
            next_line_help,
            hide,
            hide_possible_values,
            hide_default_value,
            hide_short_help,
            hide_long_help,
            conflicts_with = "other_f",
            conflicts_with_all = ["field1", "field2", "field3"],
            overrides_with = "other_field",
            overrides_with_all = ["field1", "field2", "field3"],
        ))]
        _field: bool,
    }

    let _command = Config::get_command();
}
