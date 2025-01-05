use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;

pub mod generator;

#[derive(Debug)]
pub struct Query {
    pub procedures: Vec<Procedure>,
    pub filter_procedures: Vec<FilterProcedure>
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
        for filter_procedure in &self.filter_procedures {
            write!(f, "\n")?;
            filter_procedure.fmt(f, indent)?;
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
        for instruction in &self.instructions {
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
        match self {
            Instruction::ForEachMember { .. } |
            Instruction::SaveCurrentNodeDuringTraversal { .. } => true,
            _ => false
        }
    }

    pub fn is_array_element_iteration(&self) -> bool {
        match self {
            Instruction::ForEachElement { .. } |
            Instruction::SaveCurrentNodeDuringTraversal { .. } => true,
            _ => false
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

#[derive(Debug)]
pub struct FilterProcedure {
    pub name: String,
    pub arity: usize,
    pub expression: FilterExpression,
}

impl FilterProcedure {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        let params_str = (1..self.arity + 1).into_iter()
            .map(|i| format!("param{i}"))
            .join(", ");
        let signature = format!("{}({params_str})", self.name);
        write!(f, "{} {{\n", signature)?;
        self.expression.fmt(f, indent + 1)?;
        write_indent(f, indent)?;
        write!(f, "}}\n")
    }
}

impl Display for FilterProcedure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum FilterExpression {
    Or { lhs: Box<FilterExpression>, rhs: Box<FilterExpression> },
    And { lhs: Box<FilterExpression>, rhs: Box<FilterExpression> },
    Not { expr: Box<FilterExpression> },
    Comparison { lhs: Comparable, rhs: Comparable, op: ComparisonOp },
    BoolParam { id: usize },
}

impl FilterExpression {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            FilterExpression::Or { lhs, rhs } => {
                write!(f, "Or {{\n")?;
                (**lhs).fmt(f, indent + 1)?;
                (**rhs).fmt(f, indent + 1)?;
                write_indent(f, indent)?;
                write!(f, "}}\n")
            }
            FilterExpression::And { lhs, rhs } => {
                write!(f, "And {{\n")?;
                (**lhs).fmt(f, indent + 1)?;
                (**rhs).fmt(f, indent + 1)?;
                write_indent(f, indent)?;
                write!(f, "}}\n")
            }
            FilterExpression::Not { expr } => {
                write!(f, "Not {{\n")?;
                (**expr).fmt(f, indent + 1)?;
                write_indent(f, indent)?;
                write!(f, "}}\n")
            }
            FilterExpression::Comparison { lhs, rhs, op } => {
                write!(f, "{} {{\n", op.to_string())?;
                lhs.fmt(f, indent + 1)?;
                rhs.fmt(f, indent + 1)?;
                write_indent(f, indent)?;
                write!(f, "}}\n")
            }
            FilterExpression::BoolParam { id } => write!(f, "param{id}\n")
        }
    }
}

impl Display for FilterExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum Comparable {
    Param { id: usize },
    Literal { value: LiteralValue },
}

impl Comparable {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        match self {
            Comparable::Param { id } => {
                write_indent(f, indent)?;
                write!(f, "param{id}\n")
            }
            Comparable::Literal { value } => {
                value.fmt(f, indent)
            }
        }
    }
}

impl Display for Comparable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}


#[derive(Debug)]
pub enum LiteralValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl LiteralValue {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            LiteralValue::String(str) => {
                write!(f, "\"{str}\"\n")
            }
            LiteralValue::Int(int) => {
                write!(f, "{int}\n")
            }
            LiteralValue::Float(float) => {
                write!(f, "{float}\n")
            }
            LiteralValue::Bool(bool) => {
                write!(f, "{bool}\n")
            }
            LiteralValue::Null => {
                write!(f, "null\n")
            }
        }
    }
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum ComparisonOp {
    EqualTo,
    NotEqualTo,
    LesserOrEqualTo,
    GreaterOrEqualTo,
    LessThan,
    GreaterThan,
}

impl Display for ComparisonOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

const INDENT: &str = "  ";
fn write_indent(f: &mut Formatter, indent: u16) -> fmt::Result {
    for _ in 0..indent {
        write!(f, "{}", INDENT)?;
    }
    Ok(())
}
