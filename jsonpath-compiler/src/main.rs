#![allow(dead_code)]

mod compiler;
mod ir;

use crate::compiler::compile_query;
use crate::ir::generator::generate;
use std::io::Read;
use std::process::ExitCode;

fn main() -> Result<ExitCode, std::io::Error> {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut input = String::new();
    if handle.read_to_string(&mut input)? == 0 {
        return Ok(ExitCode::FAILURE);
    }
    input = input.trim().to_owned();

    let parsing_res = rsonpath_syntax::parse(&input);
    match parsing_res {
        Ok(query_syntax) => {
            let query_ir = generate(&query_syntax);
            let target_code = compile_query(&query_ir);
            print!("{target_code}");
            return Ok(ExitCode::SUCCESS);
        }
        Err(err) => {
            println!("DBGERR: {err:?}");
            return Ok(ExitCode::FAILURE);
        }
    }
}
