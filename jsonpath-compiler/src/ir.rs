use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub mod generator;

#[derive(Debug)]
pub struct Query {
    pub procedures: Vec<Procedure>,
}

impl Query {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        let mut first = true;
        for procedure in &self.procedures {
            if !first {
                write!(f, "\n")?;
            }
            first = false;
            procedure.fmt(f, indent)?;
        }
        Ok(())
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub struct Procedure {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

impl Procedure {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        write!(f, "{} {{\n", self.name)?;
        let mut first = true;
        for instruction in &self.instructions {
            if !first {
                write!(f, "\n")?;
            }
            first = false;
            instruction.fmt(f, indent + 1)?;
        }
        write_indent(f, indent)?;
        write!(f, "}}\n")
    }
}

impl Display for Procedure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum Instruction {
    ForEachElement { instructions: Vec<Instruction> },
    ForEachMember { instructions: Vec<Instruction> },
    IfCurrentIndexEquals { index: u64, instructions: Vec<Instruction> },
    IfCurrentIndexFromEndEquals { index: u64, instructions: Vec<Instruction> },
    IfCurrentMemberNameEquals { name: String, instructions: Vec<Instruction> },
    ExecuteProcedureOnChild { name: String },
    SaveCurrentNodeDuringTraversal { instruction: Box<Instruction> },
    Continue,
    TraverseCurrentNodeSubtree
}

impl Instruction {
    pub fn is_object_member_iteration(&self) -> bool {
        if let Instruction::ForEachMember { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_array_element_iteration(&self) -> bool {
        if let Instruction::ForEachElement { .. } = self {
            true
        } else {
            false
        }
    }

    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            Instruction::ForEachElement { instructions } => {
                write!(f, "ForEachElement {{\n")?;
                for instruction in instructions {
                    instruction.fmt(f, indent + 1)?;
                }
                write_indent(f, indent)?;
                write!(f, "}}\n")
            },
            Instruction::ForEachMember { instructions } => {
                write!(f, "ForEachMember {{\n")?;
                for instruction in instructions {
                    instruction.fmt(f, indent + 1)?;
                }
                write_indent(f, indent)?;
                write!(f, "}}\n")
            },
            Instruction::IfCurrentIndexEquals { index, instructions } => {
                write!(f, "IfCurrentIndexEquals({index}) {{\n")?;
                for instruction in instructions {
                    instruction.fmt(f, indent + 1)?;
                }
                write_indent(f, indent)?;
                write!(f, "}}\n")
            },
            Instruction::IfCurrentIndexFromEndEquals { index, instructions } => {
                write!(f, "IfCurrentIndexFromEndEquals({index}) {{\n")?;
                for instruction in instructions {
                    instruction.fmt(f, indent + 1)?;
                }
                write_indent(f, indent)?;
                write!(f, "}}\n")
            },
            Instruction::IfCurrentMemberNameEquals { name, instructions } => {
                write!(f, "IfCurrentMemberNameEquals({name}) {{\n")?;
                for instruction in instructions {
                    instruction.fmt(f, indent + 1)?;
                }
                write_indent(f, indent)?;
                write!(f, "}}\n")
            },
            Instruction::ExecuteProcedureOnChild { name } => {
                write!(f, "{name}(currentChild)\n")
            },
            Instruction::SaveCurrentNodeDuringTraversal { instruction } => {
                write!(f, "SelectCurrentNode {{\n")?;
                (**instruction).fmt(f, indent + 1)?;
                write_indent(f, indent)?;
                write!(f, "}}\n")
            },
            Instruction::Continue => write!(f, "Continue\n"),
            Instruction::TraverseCurrentNodeSubtree => {
                write!(f, "TraverseCurrentNodeSubtree\n")
            }
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub struct Name(pub String);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

#[derive(Debug)]
pub struct Index(pub i64);

impl Display for Index {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

const INDENT: &str = "  ";
fn write_indent(f: &mut Formatter, indent: u16) -> fmt::Result {
    for _ in 0..indent {
        write!(f, "{}", INDENT)?;
    }
    Ok(())
}
