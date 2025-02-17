use std::fs;

use askama::Template;

use crate::ir::Instruction;
use crate::ir::Instruction::{
    ForEachElement, ForEachMember, IfCurrentIndexEquals, IfCurrentIndexFromEndEquals,
    IfCurrentMemberNameEquals,
};
use crate::NamedQuery;
use crate::targets::BindingsGenerator;

pub mod dom;
pub mod ondemand;


#[derive(Template)]
#[template(path = "simdjson/bindings.rs", escape = "none")]
struct RustBindingsTemplate {
    query_names: Vec<String>,
}

impl RustBindingsTemplate {
    pub fn new(query_names: Vec<String>) -> RustBindingsTemplate {
        RustBindingsTemplate { query_names }
    }
}

pub struct RustBindingsGenerator {
    bindings_file_path: String,
}

impl RustBindingsGenerator {
    pub fn new(bindings_file_path: &str) -> RustBindingsGenerator {
        RustBindingsGenerator {
            bindings_file_path: bindings_file_path.to_string()
        }
    }
}

impl BindingsGenerator for RustBindingsGenerator {
    fn generate(&self, named_queries: &Vec<NamedQuery>) -> Result<(), std::io::Error> {
        let query_names = named_queries.iter().map(|(name, _)| name.to_string()).collect();
        let template = RustBindingsTemplate::new(query_names);
        let bindings = template.render().unwrap();
        fs::write(&self.bindings_file_path, bindings)?;
        Ok(())
    }
}

fn is_array_length_needed(instructions: &Vec<Instruction>) -> bool {
    for instruction in instructions {
        let is_needed = match instruction {
            IfCurrentIndexFromEndEquals { .. } => true,
            IfCurrentIndexEquals {
                index: _index,
                instructions,
            } => is_array_length_needed(instructions),
            IfCurrentMemberNameEquals {
                name: _name,
                instructions,
            } => is_array_length_needed(instructions),
            ForEachMember { instructions } => is_array_length_needed(instructions),
            ForEachElement { instructions } => is_array_length_needed(instructions),
            _ => false,
        };
        if is_needed {
            return true;
        }
    }
    false
}
