use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{from_str, to_string_pretty, Value};
use uuid::Uuid;

use jsonpath_compiler::{Args, compile, DEFAULT_QUERY_NAME, generate_ir, parse_queries, Target, write_to_file};

pub struct TestHelper {
    query: String,
    document: String,
    expected_result: String,
    query_code_file_path: String,
    query_prog_file_path: String,
    document_file_path: String,
    ignore_order_and_duplicates: bool,
}

impl TestHelper {
    const WORKDIR_PATH: &'static str = "/tmp";
    const SIMDJSON_PATH: &'static str = "/opt/homebrew/Cellar/simdjson/3.11.5"; // TODO: load from the configuration

    pub fn new(query: &str, document: &str, expected_result: &str) -> TestHelper {
        let tmp_path = Self::random_file_path();
        TestHelper {
            query: query.to_string(),
            document: document.to_string(),
            query_code_file_path: format!("{tmp_path}.cpp"),
            query_prog_file_path: tmp_path.clone(),
            document_file_path: format!("{tmp_path}.json"),
            expected_result: expected_result.to_string(),
            ignore_order_and_duplicates: false,
        }
    }

    pub fn run(&self) {
        self.generate_query_code();
        self.compile_query_code();
        let result = self.normalize_result(&self.execute_query());
        let expected_result = self.normalize_result(&self.expected_result);
        assert_eq!(result, expected_result);
    }

    fn random_file_path() -> String {
        let uuid = Uuid::new_v4();
        Path::new(Self::WORKDIR_PATH).join(uuid.to_string()).to_str().unwrap().to_string()
    }

    fn ignore_order_and_duplicates(self) -> TestHelper {
        TestHelper {
            ignore_order_and_duplicates: true,
            ..self
        }
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

    fn execute_query(&self) -> String {
        fs::write(&self.document_file_path, &self.document).unwrap();
        let output = Command::new(&self.query_prog_file_path)
            .arg(&self.document_file_path)
            .output()
            .unwrap();
        assert!(output.status.success(), "query execution failed");
        String::from_utf8(output.stdout).unwrap()
    }

    fn normalize_result(&self, result: &str) -> String {
        let parsed_result: Value = from_str(result).unwrap();
        to_string_pretty(&parsed_result).unwrap()
    }
}