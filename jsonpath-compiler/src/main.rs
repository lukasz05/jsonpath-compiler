#![allow(dead_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use jsonpath_compiler::{Args, compile, DEFAULT_QUERY_NAME, generate_ir, parse_queries, read_queries_from_file, write_to_file};
use jsonpath_compiler::compiler::simdjson::RustBindingsGenerator;

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
        return Ok(ExitCode::FAILURE);
    }
    let parsed_queries = parse_result.unwrap();

    let query_irs = generate_ir(&parsed_queries, args.ir_output.clone()).unwrap();

    let code = compile(&query_irs, &args);
    if let Some(output_path) = args.output {
        write_to_file(&output_path, code)?;
        if args.rust_bindings.is_some() {
            let bindings_generator = RustBindingsGenerator::new(
                queries.iter().map(|(name, _)| name.to_string()).collect(),
            );
            let bindings_code = bindings_generator.generate();
            let mut bindings_file_path = PathBuf::from(&output_path);
            bindings_file_path.set_file_name("bindings");
            bindings_file_path.set_extension("rs");
            let bindings_file_path = bindings_file_path.to_str().unwrap();
            write_to_file(bindings_file_path, bindings_code)?;
        }
    } else {
        print!("{code}");
    }

    Ok(ExitCode::SUCCESS)
}
