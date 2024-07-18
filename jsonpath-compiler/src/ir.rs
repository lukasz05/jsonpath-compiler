use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub mod generator;

#[derive(Debug)]
pub struct Query {
    segments: Vec<Segment>,
}

impl Query {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        for segment in &self.segments {
            segment.fmt(f, indent)?;
            write!(f, "\n")?;
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
pub struct Segment {
    instructions: Vec<Instruction>,
}

impl Segment {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        write!(f, "Segment {{\n")?;
        for instruction in &self.instructions {
            instruction.fmt(f, indent + 1)?;
            write!(f, "\n")?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}


#[derive(Debug)]
pub enum Instruction {
    PushLiteral { literal: Literal },
    PushRootNode,

    PopAndPushAllChildren,
    PopAndPushChildByName { name: Name },
    PopAndPushElementAtIndex { index: Index },

    SelectAllChildren,
    SelectChildByName { name: Name },
    SelectElementAtIndex { index: Index },
    SelectSlice { slice: Slice },
    SelectNodeConditionally,

    WhileStackNotEmpty { instructions: Vec<Instruction> },

    ExistenceTest { absolute: bool, subquery: Query },
    Compare { op: ComparisonOp },
    And,
    Or,
    Not,

    FilterIteration { instructions: Vec<Instruction> },
    PushCurrentFilterNode,
}

impl Instruction {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            Instruction::PushLiteral { literal } => write!(f, "PushLiteral({literal})"),
            Instruction::PushRootNode => write!(f, "PushRootNode"),
            Instruction::PopAndPushAllChildren => write!(f, "PopAndPushAllChildren"),
            Instruction::PopAndPushChildByName { name } => write!(f, "PopAndPushChildByName({name})"),
            Instruction::PopAndPushElementAtIndex { index } => write!(f, "PopAndPushElementAtIndex({index})"),
            Instruction::SelectAllChildren => write!(f, "SelectAllChildren"),
            Instruction::SelectChildByName { name } => write!(f, "SelectChildByName({name})"),
            Instruction::SelectElementAtIndex { index } => write!(f, "SelectElementAtIndex({index})"),
            Instruction::SelectSlice { slice } => write!(f, "SelectSlice({slice})"),
            Instruction::SelectNodeConditionally => write!(f, "SelectNodeConditionally"),
            Instruction::WhileStackNotEmpty { instructions } => {
                write!(f, "WhileStackNotEmpty {{\n")?;
                for instruction in instructions {
                    instruction.fmt(f, indent + 1)?;
                    write!(f, "\n")?;
                }
                write_indent(f, indent)?;
                write!(f, "}}")
            }
            Instruction::ExistenceTest { absolute, subquery } => {
                write!(f, "ExistenceTest({}) {{\n", if *absolute { "absolute" } else { "relative" })?;
                subquery.fmt(f, indent + 1)?;
                write_indent(f, indent)?;
                write!(f, "}}")
            }
            Instruction::Compare { op } => write!(f, "Compare({op})"),
            Instruction::And => write!(f, "And"),
            Instruction::Or => write!(f, "Or"),
            Instruction::Not => write!(f, "Not"),
            Instruction::FilterIteration { instructions } => {
                write!(f, "FilterIteration {{\n")?;
                for instruction in instructions {
                    instruction.fmt(f, indent + 1)?;
                    write!(f, "\n")?;
                }
                write_indent(f, indent)?;
                write!(f, "}}")
            }
            Instruction::PushCurrentFilterNode => write!(f, "PushCurrentFilterNode")
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}


#[derive(Debug)]
pub enum Literal {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Literal::String(str) => write!(f, "\"{}\"", str),
            Literal::Int(x) => write!(f, "{}", x),
            Literal::Float(x) => write!(f, "{}", x),
            Literal::Bool(bool) => write!(f, "{}", bool),
            Literal::Null => write!(f, "null")
        }
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
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct Name(String);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

#[derive(Debug)]
pub struct Index(i64);

impl Display for Index {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Slice {
    start: i64,
    end: Option<i64>,
    step: i64,
}

impl Display for Slice {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.start, self.end.unwrap_or_default(), self.step)
    }
}


const INDENT: &str = "  ";
fn write_indent(f: &mut Formatter, indent: u16) -> fmt::Result {
    for _ in 0..indent {
        write!(f, "{}", INDENT)?;
    }
    Ok(())
}