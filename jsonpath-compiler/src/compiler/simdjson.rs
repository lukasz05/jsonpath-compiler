use askama::Template;

use crate::ir::{Instruction, Query};
use crate::ir::Instruction::{
    ForEachElement, ForEachMember, IfCurrentIndexEquals, IfCurrentIndexFromEndEquals,
    IfCurrentMemberNameEquals,
};

pub mod dom;
pub mod ondemand;

type NamedQuery<'a> = (&'a str, &'a Query);

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
    query_names: Vec<String>,
}

impl RustBindingsGenerator {
    pub fn new(query_names: Vec<String>) -> RustBindingsGenerator {
        RustBindingsGenerator { query_names }
    }

    pub fn generate(self) -> String {
        let template = RustBindingsTemplate::new(self.query_names);
        template.render().unwrap()
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
