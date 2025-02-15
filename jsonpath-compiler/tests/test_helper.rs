use std::fs;
use std::path::Path;
use std::process::Command;

use itertools::Itertools;
use serde_json::{from_str, Value};
use uuid::Uuid;

use jsonpath_compiler::{Args, compile, DEFAULT_QUERY_NAME, generate_ir, parse_queries, Target, write_to_file};

pub struct TestHelper {
    query: String,
    document: String,
    expected_result: Value,
    query_code_file_path: String,
    query_prog_file_path: String,
    document_file_path: String,
    ignore_order_and_duplicates: bool,
}

impl TestHelper {
    const WORKDIR_PATH: &'static str = "/tmp";
    const SIMDJSON_PATH: &'static str = "/opt/homebrew/Cellar/simdjson/3.12.2"; // TODO: load from the configuration

    pub fn new(query: &str, document: &str, expected_result: &str) -> TestHelper {
        let tmp_path = Self::random_file_path();
        TestHelper {
            query: query.to_string(),
            document: document.to_string(),
            query_code_file_path: format!("{tmp_path}.cpp"),
            query_prog_file_path: tmp_path.clone(),
            document_file_path: format!("{tmp_path}.json"),
            expected_result: from_str(expected_result).unwrap(),
            ignore_order_and_duplicates: false,
        }
    }

    pub fn ignore_order_and_duplicates(self) -> TestHelper {
        TestHelper {
            ignore_order_and_duplicates: true,
            ..self
        }
    }

    pub fn run(&mut self) {
        self.generate_query_code();
        self.compile_query_code();
        let mut result = self.execute_query();
        if self.ignore_order_and_duplicates {
            Self::normalize_result(&mut result);
            Self::normalize_result(&mut self.expected_result);
        }
        assert_eq!(result, self.expected_result)
    }

    fn random_file_path() -> String {
        let uuid = Uuid::new_v4();
        Path::new(Self::WORKDIR_PATH).join(uuid.to_string()).to_str().unwrap().to_string()
    }

    fn generate_query_code(&self) {
        let parse_result = parse_queries(&vec![
            (DEFAULT_QUERY_NAME.to_string(), self.query.to_string())
        ]);
        let parsed_queries = parse_result.unwrap();
        let query_irs = generate_ir(&parsed_queries, None).unwrap();
        let code = compile(&query_irs, &Args {
            standalone: true,
            target: Target::SimdjsonOndemand,
            input: self.query.to_string(),
            output: Some(self.query_code_file_path.to_string()),
            from_file: false,
            ir_output: None,
            mmap: false,
            logging: false,
            rust_bindings: None,
        });
        write_to_file(&self.query_code_file_path, code).unwrap()
    }

    fn compile_query_code(&self) {
        let result = Command::new("c++")
            .arg(&self.query_code_file_path)
            .arg("-std=c++20")
            .arg("-O3")
            .arg(format!("-I{}/include", Self::SIMDJSON_PATH))
            .arg(format!("-L{}/lib", Self::SIMDJSON_PATH))
            .arg("-lsimdjson")
            .arg("-o").arg(&self.query_prog_file_path)
            .status()
            .unwrap();
        assert!(result.success(), "query code compilation failed")
    }

    fn execute_query(&self) -> Value {
        fs::write(&self.document_file_path, &self.document).unwrap();
        let output = Command::new(&self.query_prog_file_path)
            .arg(&self.document_file_path)
            .output()
            .unwrap();
        assert!(output.status.success(), "query execution failed");
        let result = String::from_utf8(output.stdout).unwrap();
        from_str(&result).unwrap()
    }

    fn normalize_result(result: &mut Value) {
        let result_arr = result.as_array_mut().unwrap();
        result_arr.sort_by_key(|v| v.to_string());
        result_arr.dedup()
    }
}