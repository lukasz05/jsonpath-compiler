use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use rsonpath_syntax::JsonPathQuery;

use crate::compiler::simdjson::dom::ToDomCompiler;
use crate::compiler::simdjson::ondemand::ToOnDemandCompiler;
use crate::ir::generator::IRGenerator;
use crate::ir::Query;

pub mod compiler;
mod ir;

pub const DEFAULT_QUERY_NAME: &str = "query";

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Target {
    SimdjsonOndemand,
    SimdjsonDom,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// JSONPath query or a file with queries to be compiled.
    pub input: String,

    #[arg(long)]
    pub target: Target,

    /// Treat the input as a path to a file with queries.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub from_file: bool,

    /// File in which to place the query code.
    /// If not specified, the code is written to stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    /// File in which to place the intermediate query code.
    #[arg(long)]
    pub ir_output: Option<String>,

    /// Generate code of a standalone program which executes the query.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub standalone: bool,

    /// Memory-map the input document during query execution.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub mmap: bool,

    // Print logs during query execution.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub logging: bool,

    // File in which to place Rust bindings for query procedures.
    #[arg(long, action)]
    pub rust_bindings: Option<String>,
}

pub fn write_to_file(file_path: &str, content: String) -> Result<(), std::io::Error> {
    let file_path = PathBuf::from(file_path);
    if let Some(p) = file_path.parent() {
        fs::create_dir_all(p)?
    };
    fs::write(file_path, content)
}

type NamedRawQuery = (String, String);
type NamedParsedQuery = (String, JsonPathQuery);
type NamedQuery = (String, Query);

pub fn read_queries_from_file(file_path: &str) -> Result<Vec<NamedRawQuery>, std::io::Error> {
    let mut queries = Vec::new();
    let query_file = File::open(file_path)?;
    for line in BufReader::new(query_file).lines().flatten() {
        let (name, query) = line.split_once(" ").unwrap();
        queries.push((name.to_string(), query.to_string()));
    }
    Ok(queries)
}

pub fn parse_queries(
    queries: &Vec<NamedRawQuery>,
) -> Result<Vec<NamedParsedQuery>, Vec<(String, rsonpath_syntax::error::ParseError)>> {
    let parse_results: Vec<(String, rsonpath_syntax::Result<JsonPathQuery>)> = queries
        .iter()
        .map(|(name, query)| (name.to_string(), rsonpath_syntax::parse(query)))
        .collect();
    let mut parse_errors = Vec::new();
    let mut parsed_queries = Vec::new();
    for (name, parse_result) in parse_results {
        match parse_result {
            Ok(query_syntax) => parsed_queries.push((name.to_string(), query_syntax)),
            Err(err) => parse_errors.push((name.to_string(), err)),
        }
    }
    return if !parse_errors.is_empty() {
        Err(parse_errors)
    } else {
        Ok(parsed_queries)
    };
}

pub fn generate_ir<'a>(
    parsed_queries: &Vec<NamedParsedQuery>,
    ir_output_path: Option<String>,
) -> Result<Vec<NamedQuery>, std::io::Error> {
    let mut query_irs = Vec::new();
    for (name, parsed_query) in parsed_queries.to_owned() {
        let ir_generator = IRGenerator::new(&parsed_query);
        let query_ir = ir_generator.generate();
        query_irs.push((name, query_ir));
    }
    if let Some(ir_output_path) = ir_output_path {
        write_ir_to_file(&query_irs, ir_output_path)?;
    }
    Ok(query_irs)
}

pub fn compile(queries: &Vec<NamedQuery>, args: &Args) -> String {
    match args.target {
        Target::SimdjsonOndemand => {
            let compiler = if args.standalone {
                let (name, query) = queries.first().unwrap();
                ToOnDemandCompiler::new_standalone((name, query), args.logging, args.mmap)
            } else {
                let filename = args.output.clone().map(|f| {
                    Path::new(&f)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                });
                ToOnDemandCompiler::new_lib(
                    queries
                        .iter()
                        .map(|(name, query_ir)| (name.as_str(), query_ir))
                        .collect(),
                    args.logging,
                    args.rust_bindings.is_some(),
                    filename,
                )
            };
            compiler.compile()
        }
        Target::SimdjsonDom => {
            let compiler = if args.standalone {
                let (name, query) = queries.first().unwrap();
                ToDomCompiler::new_standalone((name, query), args.logging, args.mmap)
            } else {
                let filename = args.output.clone().map(|f| {
                    Path::new(&f)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                });
                ToDomCompiler::new_lib(
                    queries
                        .iter()
                        .map(|(name, query_ir)| (name.as_str(), query_ir))
                        .collect(),
                    args.logging,
                    args.rust_bindings.is_some(),
                    filename,
                )
            };
            compiler.compile()
        }
    }
}

pub fn write_ir_to_file(
    query_irs: &Vec<NamedQuery>,
    ir_output_path: String,
) -> Result<(), std::io::Error> {
    let mut ir_file = File::create(ir_output_path).unwrap();
    for (name, query_ir) in query_irs {
        write!(&mut ir_file, "{name}:\n")?;
        write!(&mut ir_file, "{:#?}", query_ir)?;
        write!(&mut ir_file, "\n")?;
    }
    Ok(())
}