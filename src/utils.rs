// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::{ConfigOption, Source};

pub(super) fn parse_subcommand<T>(
    args: impl Iterator<Item = String>,
    am: &clap::ArgMatches,
) -> Result<Option<T>, crate::Error>
where
    T: clap::Subcommand,
{
    let subname = match am.subcommand_name() {
        None => return Ok(None),
        Some(name) => name,
    };
    let vars = args.collect::<Vec<_>>();
    let (_, subcommand) = vars.split_at(
        vars.iter()
            .position(|arg| arg.to_lowercase() == subname)
            .ok_or_else(|| {
                crate::Error::ExternalError("not found subcommand name in std::env::args()".into())
            })?,
    );
    let subcommand = [&["".to_string()], subcommand].concat();

    #[derive(clap::Parser)]
    struct Supporting<S: clap::Subcommand> {
        #[clap(subcommand)]
        inner: S,
    }

    <Supporting<T> as clap::Parser>::try_parse_from(subcommand)
        .map(|supporting| Some(supporting.inner))
        .map_err(|err| {
            crate::Error::ExternalError(format!("wrong parsing in parse_subcommand: {err}"))
        })
}

pub(super) fn find_field_in_table(
    config: &HashMap<String, config::Value>,
    table: Option<String>,
    field_name: String,
) -> Result<Option<String>, crate::Error> {
    let mut field_segs = deconstruct_table_path(&field_name).collect::<Vec<_>>();
    let field = field_segs.pop().ok_or_else(|| {
        crate::Error::FailedParse(format!("Empty path segments of the field: {field_name}"))
    })?;

    let sub_config = match table {
        None => match find_sub_table(config, field_segs.into_iter())? {
            None => return Ok(None),
            Some(sub_config) => sub_config,
        },
        Some(table) => match find_sub_table(
            config,
            deconstruct_table_path(&table).chain(field_segs.into_iter()),
        )? {
            None => return Ok(None),
            Some(sub_config) => sub_config,
        },
    };

    if let Some(value) = sub_config.get(&field) {
        from_config_to_string(value.clone()).map(Some)
    } else {
        Ok(None)
    }
}

fn find_sub_table(
    parent_config: &HashMap<String, config::Value>,
    mut table: impl Iterator<Item = String>,
) -> Result<Option<&HashMap<String, config::Value>>, crate::Error> {
    let first_segment = match table.next() {
        None => return Ok(Some(parent_config)),
        Some(seg) => seg,
    };

    match &parent_config.get(&first_segment) {
        None => Ok(None),
        Some(sub_table) => {
            if let config::ValueKind::Table(sub_table) = &sub_table.kind {
                find_sub_table(sub_table, table)
            } else {
                Err(crate::Error::FailedParse(format!(
                    "Field {first_segment} is found in configuration files but it is not a table"
                )))
            }
        }
    }
}

fn deconstruct_table_path(table: &str) -> impl Iterator<Item = String> + '_ {
    table.split(|dot| dot == '.').map(ToString::to_string)
}

fn from_config_to_string(initial: config::Value) -> Result<String, super::Error> {
    fn from_config_to_serde_json(
        initial: config::ValueKind,
    ) -> Result<serde_json::Value, super::Error> {
        match initial {
            config::ValueKind::Nil => Ok(serde_json::Value::Null),
            config::ValueKind::Boolean(b) => Ok(serde_json::Value::Bool(b)),
            config::ValueKind::I64(i) => Ok(serde_json::Value::Number(
                serde_json::value::Number::from(i),
            )),
            config::ValueKind::U64(u) => Ok(serde_json::Value::Number(
                serde_json::value::Number::from(u),
            )),
            config::ValueKind::Float(f) => Ok(serde_json::Value::Number(
                serde_json::value::Number::from_f64(f).ok_or_else(|| {
                    super::Error::FailedParse(format!(
                        "failed to convert to serde_json Number from from f64 {}",
                        f
                    ))
                })?,
            )),
            config::ValueKind::String(s) => Ok(serde_json::Value::String(s)),
            config::ValueKind::Table(tbl) => {
                Ok(serde_json::Value::Object(serde_json::Map::from_iter({
                    let mut res = vec![];
                    for (k, v) in tbl {
                        res.push((k, from_config_to_serde_json(v.kind)?));
                    }
                    res.into_iter()
                })))
            }
            config::ValueKind::Array(arr) => Ok(serde_json::Value::Array({
                let mut res = vec![];
                for v in arr {
                    res.push(from_config_to_serde_json(v.kind)?);
                }
                res
            })),
            config::ValueKind::I128(num) => Ok(serde_json::Value::Number(
                ::std::str::FromStr::from_str(&num.to_string()).map_err(|_| {
                    super::Error::FailedParse(format!(
                        "can't convert to serde_json::value::Number from I128, value: {})",
                        num
                    ))
                })?,
            )),
            config::ValueKind::U128(num) => Ok(serde_json::Value::Number(
                ::std::str::FromStr::from_str(&num.to_string()).map_err(|_| {
                    super::Error::FailedParse(format!(
                        "can't convert to serde_json::value::Number from U128, value: {})",
                        num
                    ))
                })?,
            )),
        }
    }
    match initial.kind {
        config::ValueKind::I128(num) => Ok(num.to_string()),
        config::ValueKind::U128(num) => Ok(num.to_string()),
        value => Ok(from_config_to_serde_json(value).map(|value| value.to_string())?),
    }
}

impl PartialEq for ConfigOption {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ConfigOption::EnvPrefix(_), ConfigOption::EnvPrefix(_))
                | (
                    ConfigOption::ExplicitSource(Source::Clap(_)),
                    ConfigOption::ExplicitSource(Source::Clap(_)),
                )
                | (
                    ConfigOption::ExplicitSource(Source::ConfigFiles(_)),
                    ConfigOption::ExplicitSource(Source::ConfigFiles(_)),
                )
                | (
                    ConfigOption::ExplicitSource(Source::Env(_)),
                    ConfigOption::ExplicitSource(Source::Env(_)),
                )
        )
    }
}
impl Eq for ConfigOption {}

impl Hash for ConfigOption {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ConfigOption::EnvPrefix(_) => state.write_u8(1),
            ConfigOption::ExplicitSource(Source::Clap(_)) => state.write_u8(2),
            ConfigOption::ExplicitSource(Source::ConfigFiles(_)) => state.write_u8(3),
            ConfigOption::ExplicitSource(Source::Env(_)) => state.write_u8(4),
        }
    }
}
