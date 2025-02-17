use std::{env, fs};
use std::path::Path;
use std::process::Command;

use serde_json::{from_str, Value};
use uuid::Uuid;

use jsonpath_compiler::compiler::StandaloneProgGeneratingCompiler;
use jsonpath_compiler::Target;
use jsonpath_compiler::targets::simdjson::dom::DomCodeStandaloneProgGenerator;
use jsonpath_compiler::targets::simdjson::ondemand::OnDemandCodeStandaloneProgGenerator;

pub struct TestHelper {
    query: String,
    document: String,
    expected_result: Value,
    target: Target,
    simdjson_path: String,
    query_code_file_path: String,
    query_prog_file_path: String,
    document_file_path: String,
    ignore_order_and_duplicates: bool,
}

impl TestHelper {
    const WORKDIR_PATH: &'static str = "/tmp";

    pub fn new(query: &str, document: &str, expected_result: &str, target: Target) -> TestHelper {
        let tmp_path = Self::random_file_path();
        TestHelper {
            query: query.to_string(),
            document: document.to_string(),
            simdjson_path: env::var("SIMDJSON_PATH").unwrap(),
            query_code_file_path: format!("{tmp_path}.cpp"),
            query_prog_file_path: tmp_path.clone(),
            document_file_path: format!("{tmp_path}.json"),
            expected_result: from_str(expected_result).unwrap(),
            ignore_order_and_duplicates: false,
            target,
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
        let compiler = StandaloneProgGeneratingCompiler::new();
        match self.target {
            Target::SimdjsonOndemand => {
                compiler.compile::<OnDemandCodeStandaloneProgGenerator>(
                    &self.query,
                    &self.query_code_file_path,
                )
            }
            Target::SimdjsonDom => {
                compiler.compile::<DomCodeStandaloneProgGenerator>(
                    &self.query,
                    &self.query_code_file_path,
                )
            }
        }.unwrap();
    }

    fn compile_query_code(&self) {
        let result = Command::new("c++")
            .arg(&self.query_code_file_path)
            .arg("-std=c++20")
            .arg("-O3")
            .arg(format!("-I{}/include", self.simdjson_path))
            .arg(format!("-L{}/lib", self.simdjson_path))
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