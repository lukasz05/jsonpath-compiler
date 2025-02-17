use crate::ir::Query;
use crate::NamedQuery;

pub mod simdjson;

pub trait TargetCodeGenerator {
    fn base(&self) -> &TargetCodeGeneratorBase;

    fn logging(&self) -> bool {
        self.base().logging
    }

    fn generate(&self) -> String;
}

pub trait TargetCodeStandaloneProgGenerator: TargetCodeGenerator {
    fn new(query: Query, logging: bool, mmap: bool) -> impl TargetCodeStandaloneProgGenerator;

    fn base(&self) -> &TargetCodeStandaloneProgGeneratorBase;

    fn mmap(&self) -> bool {
        TargetCodeStandaloneProgGenerator::base(self).mmap
    }

    fn query(&self) -> &Query {
        &TargetCodeStandaloneProgGenerator::base(self).query
    }
}

pub trait TargetCodeLibGenerator: TargetCodeGenerator {
    fn new(
        named_queries: Vec<NamedQuery>,
        filename: String,
        logging: bool,
        bindings: bool,
    ) -> impl TargetCodeLibGenerator;

    fn base(&self) -> &TargetCodeLibGeneratorBase;

    fn queries(&self) -> &Vec<NamedQuery> {
        &TargetCodeLibGenerator::base(self).queries
    }

    fn filename(&self) -> &str {
        &TargetCodeLibGenerator::base(self).filename
    }

    fn bindings(&self) -> bool {
        TargetCodeLibGenerator::base(self).bindings
    }
}

pub trait BindingsGenerator {
    fn generate(&self, named_queries: &Vec<NamedQuery>) -> Result<(), std::io::Error>;
}

pub struct TargetCodeGeneratorBase {
    logging: bool,
}

impl TargetCodeGeneratorBase {
    pub fn new(logging: bool) -> TargetCodeGeneratorBase {
        TargetCodeGeneratorBase { logging }
    }
}

pub struct TargetCodeStandaloneProgGeneratorBase {
    base: TargetCodeGeneratorBase,
    mmap: bool,
    query: Query,
}

impl TargetCodeStandaloneProgGeneratorBase {
    pub fn new(query: Query, logging: bool, mmap: bool) -> TargetCodeStandaloneProgGeneratorBase {
        TargetCodeStandaloneProgGeneratorBase {
            base: TargetCodeGeneratorBase::new(logging),
            mmap,
            query,
        }
    }
}

pub struct TargetCodeLibGeneratorBase {
    base: TargetCodeGeneratorBase,
    filename: String,
    bindings: bool,
    queries: Vec<NamedQuery>,
}

impl TargetCodeLibGeneratorBase {
    pub fn new(
        named_queries: Vec<NamedQuery>,
        filename: String,
        logging: bool,
        bindings: bool,
    ) -> TargetCodeLibGeneratorBase {
        TargetCodeLibGeneratorBase {
            base: TargetCodeGeneratorBase::new(logging),
            filename,
            bindings,
            queries: named_queries,
        }
    }
}