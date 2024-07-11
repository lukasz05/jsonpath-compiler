#![allow(dead_code)]

mod ir;

use std::process::ExitCode;
use crate::ir::generator::generate;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("provide exactly one argument, the query string");
        return ExitCode::FAILURE;
    }

    let input: &str = &args[1];

    let res = rsonpath_syntax::parse(input);

    match res {
        Ok(query_syntax) => {
            let query_ir = generate(&query_syntax);
            // for segment in query_ir.segments() {
            //     print!("{segment:?}\n");
            // }
            print!("{query_ir:?}");
        },

        Err(err) => {
            println!("DBGERR: {err:?}");
        }
    }

    ExitCode::SUCCESS
}