use clap::ValueEnum;
use rsonpath_syntax::JsonPathQuery;

use crate::ir::Query;

pub mod targets;
pub mod compiler;
mod ir;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Target {
    SimdjsonOndemand,
    SimdjsonDom,
}

type NamedRawQuery = (String, String);
type NamedParsedQuery = (String, JsonPathQuery);
type NamedQuery = (String, Query);