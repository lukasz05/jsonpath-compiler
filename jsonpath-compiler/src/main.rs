#![allow(dead_code)]

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::ExitCode;

use clap::Parser;
use rsonpath_syntax::JsonPathQuery;

use crate::compiler::ToOnDemandCompiler;
use crate::ir::generator::IRGenerator;
use crate::ir::Query;

mod compiler;
mod ir;

const DEFAULT_QUERY_NAME: &str = "query";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// JSONPath query or a file with queries to be compiled.
    input: String,

    /// Treat the input as a path to a file with queries.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    from_file: bool,

    /// File in which to place the query code.
    /// If not specified, the code is written to stdout.
    #[arg(short, long)]
    output: Option<String>,

    /// File in which to place the intermediate query code.
    #[arg(long)]
    ir_output: Option<String>,

    /// Generate code of a standalone program which executes the query.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    standalone: bool,

    /// Memory-map the input document during query execution.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    mmap: bool,

    // Print logs during query execution.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    logging: bool
}

fn main() -> Result<ExitCode, std::io::Error> {
    let args = Args::parse();
    let input = args.input.trim();

    let queries = if args.from_file {
        read_queries_from_file(input)?
    } else {
        vec![(DEFAULT_QUERY_NAME.to_string(), input.to_string())]
    };

    if args.standalone && queries.len() > 1 {
        println!("Multiple queries are not allowed in the standalone mode.");
        return Ok(ExitCode::FAILURE);
    }

    let parse_result = parse_queries(&queries);
    if let Err(parse_errors) = parse_result {
        for (name, parse_error) in parse_errors {
            if name != DEFAULT_QUERY_NAME {
                print!("{name}: ");
            }
            println!("{parse_error:?}");
        }
        return Ok(ExitCode::FAILURE)
    }
    let parsed_queries = parse_result.unwrap();

    let query_irs = generate_ir(&parsed_queries, args.ir_output.clone()).unwrap();

    let code = compile(&query_irs, &args);
    if let Some(output_path) = args.output {
        let mut file = File::create(output_path)?;
        file.write_all(code.as_bytes())?;
    } else {
        print!("{code}");
    }

    Ok(ExitCode::SUCCESS)
}

type NamedRawQuery = (String, String);
type NamedParsedQuery = (String, JsonPathQuery);
type NamedQuery = (String, Query);

fn read_queries_from_file(file_path: &str) -> Result<Vec<NamedRawQuery>, std::io::Error> {
    let mut queries = Vec::new();
    let query_file = File::open(file_path)?;
    for line in BufReader::new(query_file).lines().flatten() {
        let (name, query) = line.split_once(" ").unwrap();
        queries.push((name.to_string(), query.to_string()));
    }
    Ok(queries)
}

fn parse_queries(
    queries: &Vec<NamedRawQuery>
) -> Result<Vec<NamedParsedQuery>, Vec<(String, rsonpath_syntax::error::ParseError)>> {
    let parse_results: Vec<(String, rsonpath_syntax::Result<JsonPathQuery>)> = queries.iter()
        .map(|(name, query)| (name.to_string(), rsonpath_syntax::parse(query)))
        .collect();
    let mut parse_errors = Vec::new();
    let mut parsed_queries = Vec::new();
    for (name, parse_result) in parse_results {
        match parse_result {
            Ok(query_syntax) => parsed_queries.push((name, query_syntax)),
            Err(err) => parse_errors.push((name, err))
        }
    }
    return if !parse_errors.is_empty() {
        Err(parse_errors)
    } else {
        Ok(parsed_queries)
    }
}

fn generate_ir(
    parsed_queries: &Vec<NamedParsedQuery>,
    ir_output_path: Option<String>,
) -> Result<Vec<NamedQuery>, std::io::Error> {
    let mut query_irs = Vec::new();
    for (name, parsed_query) in parsed_queries {
        let ir_generator = IRGenerator::new(&parsed_query);
        let query_ir = ir_generator.generate();
        query_irs.push((name.to_string(), query_ir));
    }
    if let Some(ir_output_path) = ir_output_path {
        write_ir_to_file(&query_irs, ir_output_path)?;
    }
    Ok(query_irs)
}

fn compile(
    queries: &Vec<NamedQuery>,
    args: &Args,
) -> String {
    let compiler = if args.standalone {
        let (name, query) = queries.first().unwrap();
        ToOnDemandCompiler::new_standalone(
            (name.to_string(), query),
            args.logging,
            args.mmap,
            args.output.clone(),
        )
    } else {
        ToOnDemandCompiler::new_header(
            queries.iter().map(|(name, query_ir)| (name.to_string(), query_ir)).collect(),
            args.logging,
            args.output.clone(),
        )
    };
    compiler.compile()
}

fn write_ir_to_file(
    query_irs: &Vec<NamedQuery>,
    ir_output_path: String,
) -> Result<(), std::io::Error> {
    let mut ir_file = File::create(ir_output_path).unwrap();
    for (name, query_ir) in query_irs {
        write!(&mut ir_file, "{name}:\n")?;
        write!(&mut ir_file, "{}", query_ir)?;
        write!(&mut ir_file, "\n")?;
    }
    Ok(())
}
