use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{NamedParsedQuery, NamedQuery, NamedRawQuery};
use crate::ir::generator::IRGenerator;
use crate::ir::Query;
use crate::targets::{BindingsGenerator, TargetCodeGenerator, TargetCodeLibGenerator,
                     TargetCodeStandaloneProgGenerator};

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] rsonpath_syntax::error::ParseError),
    #[error("multiple compilation errors")]
    MultipleErrors(Vec<(String, CompilationError)>),
}

struct CompilerHelper {}

impl CompilerHelper {
    fn parse(
        query: &str
    ) -> Result<rsonpath_syntax::JsonPathQuery, CompilationError> {
        rsonpath_syntax::parse(query).map_err(|err| CompilationError::ParseError(err))
    }

    fn generate_ir(parsed_query: &rsonpath_syntax::JsonPathQuery) -> Query {
        let ir_generator = IRGenerator::new(&parsed_query);
        ir_generator.generate()
    }

    fn generate_query_ir_output(query_ir: &Query) -> String {
        format!("{:#?}\n", query_ir)
    }

    fn generate_queries_irs_output(queries_irs: &Vec<NamedQuery>) -> String {
        let mut output = String::new();
        for (name, query_ir) in queries_irs {
            output.push_str(&format!("{name}:\n"));
            output.push_str(&Self::generate_query_ir_output(query_ir));
        }
        output
    }

    fn write_to_file(file_path: &str, content: String) -> Result<(), CompilationError> {
        let file_path = PathBuf::from(file_path);
        if let Some(p) = file_path.parent() {
            fs::create_dir_all(p)?
        };
        fs::write(file_path, content).map_err(|err| CompilationError::IoError(err))
    }
}

pub struct StandaloneProgGeneratingCompiler {
    logging: bool,
    mmap: bool,
    ir_output_file_path: Option<String>
}

impl StandaloneProgGeneratingCompiler {
    pub fn new() -> StandaloneProgGeneratingCompiler {
        StandaloneProgGeneratingCompiler {
            logging: false,
            mmap: false,
            ir_output_file_path: None
        }
    }

    pub fn with_logging(self) -> StandaloneProgGeneratingCompiler {
        StandaloneProgGeneratingCompiler {
            logging: true,
            ..self
        }
    }

    pub fn with_mmap(self) -> StandaloneProgGeneratingCompiler {
        StandaloneProgGeneratingCompiler {
            mmap: true,
            ..self
        }
    }

    pub fn write_ir_to_file(self, file_path: &str) -> StandaloneProgGeneratingCompiler {
        StandaloneProgGeneratingCompiler {
            ir_output_file_path: Some(file_path.to_string()),
            ..self
        }
    }

    pub fn compile<T: TargetCodeStandaloneProgGenerator>(
        self,
        query: &str,
        output_file_path: &str,
    ) -> Result<(), CompilationError> {
        let parsed_query = CompilerHelper::parse(query)?;
        let query_ir = CompilerHelper::generate_ir(&parsed_query);
        if let Some(ir_output_file_path) = self.ir_output_file_path {
            let ir_output = CompilerHelper::generate_query_ir_output(&query_ir);
            CompilerHelper::write_to_file(&ir_output_file_path, ir_output)?;
        }
        let target_code_generator = T::new(
            query_ir,
            self.logging,
            self.mmap
        );
        let target_code = target_code_generator.generate();
        CompilerHelper::write_to_file(output_file_path, target_code)
    }
}


pub struct LibGeneratingCompiler {
    logging: bool,
    bindings_generator: Option<Box<dyn BindingsGenerator>>,
    ir_output_file_path: Option<String>,
}

impl LibGeneratingCompiler {
    pub fn new() -> LibGeneratingCompiler {
        LibGeneratingCompiler {
            logging: false,
            bindings_generator: None,
            ir_output_file_path: None,
        }
    }

    pub fn add_bindings_generator(self, bindings_generator: impl BindingsGenerator + 'static) -> LibGeneratingCompiler {
        LibGeneratingCompiler {
            bindings_generator: Some(Box::new(bindings_generator)),
            ..self
        }
    }

    pub fn with_logging(self) -> LibGeneratingCompiler {
        LibGeneratingCompiler {
            logging: true,
            ..self
        }
    }

    pub fn write_ir_to_file(self, file_path: &str) -> LibGeneratingCompiler {
        LibGeneratingCompiler {
            ir_output_file_path: Some(file_path.to_string()),
            ..self
        }
    }

    pub fn compile<T: TargetCodeLibGenerator>(
        self,
        queries: QueriesSource,
        output_file_path: &str,
    ) -> Result<(), CompilationError> {
        let queries = match queries {
            QueriesSource::Immediate { queries } => queries,
            QueriesSource::File { file_path } => Self::read_queries_from_file(&file_path)?
        };
        let parsed_queries = Self::parse_queries(&queries)?;
        let queries_irs = Self::generate_ir(&parsed_queries);
        if let Some(ir_output_file_path) = self.ir_output_file_path {
            let ir_output = CompilerHelper::generate_queries_irs_output(&queries_irs);
            CompilerHelper::write_to_file(&ir_output_file_path, ir_output)?;
        }
        let filename = Path::new(output_file_path).file_name().unwrap().to_str().unwrap()
            .to_string();
        let bindings = self.bindings_generator.is_some();
        if let Some(bindings_generator) = self.bindings_generator {
            bindings_generator.generate(&queries_irs)
                .map_err(|err| CompilationError::IoError(err))?;
        };
        let target_code_generator = T::new(
            queries_irs,
            filename,
            self.logging,
            bindings,
        );
        let target_code = target_code_generator.generate();
        CompilerHelper::write_to_file(output_file_path, target_code)?;
        Ok(())
    }

    fn read_queries_from_file(file_path: &str) -> Result<Vec<NamedRawQuery>, CompilationError> {
        let mut queries = Vec::new();
        let query_file = File::open(file_path)?;
        for line in BufReader::new(query_file).lines().flatten() {
            let (name, query) = line.split_once(" ").unwrap();
            queries.push((name.to_string(), query.to_string()));
        }
        Ok(queries)
    }

    fn parse_queries(
        queries: &Vec<NamedRawQuery>,
    ) -> Result<Vec<NamedParsedQuery>, CompilationError> {
        let parse_results: Vec<(String, Result<rsonpath_syntax::JsonPathQuery, CompilationError>)> =
            queries.iter()
                .map(|(name, query)| (name.to_string(), CompilerHelper::parse(query)))
                .collect();
        let mut parse_errors = Vec::new();
        let mut parsed_queries = Vec::new();
        for (name, parse_result) in parse_results {
            match parse_result {
                Ok(query_syntax) => parsed_queries.push((name.to_string(), query_syntax)),
                Err(err) => {
                    parse_errors.push((name.to_string(), err))
                }
            }
        }
        return if !parse_errors.is_empty() {
            Err(CompilationError::MultipleErrors(parse_errors))
        } else {
            Ok(parsed_queries)
        };
    }

    fn generate_ir(queries: &Vec<NamedParsedQuery>) -> Vec<NamedQuery> {
        queries.iter()
            .map(|(name, query)| (name.to_string(), CompilerHelper::generate_ir(query)))
            .collect()
    }
}

pub enum QueriesSource {
    File { file_path: String },
    Immediate { queries: Vec<NamedRawQuery> },
}