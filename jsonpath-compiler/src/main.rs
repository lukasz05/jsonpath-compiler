#![allow(dead_code)]

mod ir;

use std::io::BufRead;
use crate::ir::generator::generate;

fn main() -> Result<(), std::io::Error> {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let mut line = String::new();
    loop {
        line.clear();
        if handle.read_line(&mut line)? == 0 {
            break;
        }
        line = line.trim().to_owned();

        let parsing_res = rsonpath_syntax::parse(&line);
        match parsing_res {
            Ok(query_syntax) => {
                let query_ir = generate(&query_syntax);
                print!("{query_ir}");
            }
            Err(err) => {
                println!("DBGERR: {err:?}");
            }
        }
    }
    Ok(())
}