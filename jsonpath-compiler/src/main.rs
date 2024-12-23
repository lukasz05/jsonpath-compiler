#![allow(dead_code)]

use std::fs::File;
use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use crate::compiler::ToOnDemandCompiler;
use crate::ir::generator::IRGenerator;

mod compiler;
mod ir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// JSONPath query to be compiled.
    query: String,

    /// File in which to place the query code.
    /// If not specified, the code is written to stdout.
    #[arg(short, long)]
    output: Option<String>,

    /// File in which to place the intermediate query code.
    #[arg(long)]
    ir_output: Option<String>,

    /// Memory-map the input document during query execution.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    mmap: bool,

    // Print logs during query execution.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    logging: bool
}

fn main() -> Result<ExitCode, std::io::Error> {
    let args = Args::parse();
    let input = args.query.trim();

    let parsing_res = rsonpath_syntax::parse(&input);
    return match parsing_res {
        Ok(query_syntax) => {
            let ir_generator = IRGenerator::new(&query_syntax);
            let query_ir = ir_generator.generate();
            if let Some(ir_output_path) = args.ir_output {
                let mut file = File::create(ir_output_path)?;
                file.write_all(query_ir.to_string().as_bytes())?;
            }
            let compiler = ToOnDemandCompiler::new(&query_ir, args.logging, args.mmap);
            let code = compiler.compile();
            if let Some(output_path) = args.output {
                let mut file = File::create(output_path)?;
                file.write_all(code.as_bytes())?;
            } else {
                print!("{code}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(err) => {
            println!("DBGERR: {err:?}");
            Ok(ExitCode::FAILURE)
        }
    }
}
