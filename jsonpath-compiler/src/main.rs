#![allow(dead_code)]

use std::io::Read;
use std::process::ExitCode;

use crate::compiler::ToOnDemandCompiler;
use crate::ir::generator::IRGenerator;

mod compiler;
mod ir;

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
            let ir_generator = IRGenerator::new(&query_syntax);
            let query_ir = ir_generator.generate();
            //print!("{query_ir}\n");

            let compiler = ToOnDemandCompiler::new(&query_ir);
            let code = compiler.compile();
            print!("{code}");

            return Ok(ExitCode::SUCCESS);
        }
        Err(err) => {
            println!("DBGERR: {err:?}");
            return Ok(ExitCode::FAILURE);
        }
    }
}
