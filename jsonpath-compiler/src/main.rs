use std::process::ExitCode;

use clap::Parser;

use jsonpath_compiler::compiler::{CompilationError, LibGeneratingCompiler, StandaloneProgGeneratingCompiler, QueriesSource};
use jsonpath_compiler::Target;
use jsonpath_compiler::targets::simdjson::dom::{DomCodeLibGenerator, DomCodeStandaloneProgGenerator};
use jsonpath_compiler::targets::simdjson::ondemand::{OnDemandCodeLibGenerator, OnDemandCodeStandaloneProgGenerator};
use jsonpath_compiler::targets::simdjson::RustBindingsGenerator;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// JSONPath query or a file with queries to be compiled.
    pub input: String,

    #[arg(long)]
    pub target: Target,

    /// File in which to place the query code.
    #[arg(short, long)]
    pub output: String,

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

fn main() -> Result<ExitCode, CompilationError> {
    let args = Args::parse();
    let input = args.input.trim();
    if args.standalone {
        let mut compiler = StandaloneProgGeneratingCompiler::new();
        if args.logging {
            compiler = compiler.with_logging();
        }
        if args.mmap {
            compiler = compiler.with_mmap();
        }
        match args.target {
            Target::SimdjsonOndemand => {
                compiler.compile::<OnDemandCodeStandaloneProgGenerator>(input, &args.output)
            }
            Target::SimdjsonDom => {
                compiler.compile::<DomCodeStandaloneProgGenerator>(input, &args.output)
            }
        }?;
    } else {
        let queries = QueriesSource::File { file_path: input.to_string() };
        let mut compiler = LibGeneratingCompiler::new();
        if let Some(bindings_file_path) = args.rust_bindings {
            compiler = compiler.add_bindings_generator(
                RustBindingsGenerator::new(&bindings_file_path)
            );
        }
        match args.target {
            Target::SimdjsonOndemand => {
                compiler.compile::<OnDemandCodeLibGenerator>(queries, &args.output)
            }
            Target::SimdjsonDom => {
                compiler.compile::<DomCodeLibGenerator>(queries, &args.output)
            }
        }?;
    }
    Ok(ExitCode::SUCCESS)
}
