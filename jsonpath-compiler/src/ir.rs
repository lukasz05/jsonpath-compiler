use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub mod generator;

/// An intermediate representation of a JSONPath query.
/// It consists of segments, which correspond to the segments of the original query.
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


/// Each segment is a sequence of instructions which is executed for each node of its input nodelist.
/// Instructions operate on a stack which is initialized with the current input node before each
/// execution. Instructions can add nodes to the segment's output nodelist.
/// After the segment has been executed for each input node, its output nodelist becomes an input
/// nodelist for the next segment. The input nodelist of the first segment consists of the root node
/// of the query argument. The output nodelist of the last segment is the result of the query.
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
    /// Pushes the given literal value onto the stack.
    PushLiteral { literal: Literal },
    /// Pushes the root node onto the stack.
    PushRootNode,
    /// Pops a node from the stack and pushes all its children (elements/members) onto the stack.
    PopAndPushAllChildren,
    /// Pops a node from the stack and pushes its member with the given name onto the stack.
    /// If the popped node is not an object, or it does not have a member with the given name,
    /// a special null node is pushed onto the stack.
    PopAndPushChildByName { name: Name },
    /// Pops a node from the stack and pushes its element at the given index onto the stack.
    /// If the popped node is not an array, or it does not have the given index, a special null node
    /// is pushed onto the stack.
    PopAndPushElementAtIndex { index: Index },
    /// Adds all children (elements/members) of the node at the top of the stack to the current
    /// segment's output nodelist.
    SelectAllChildren,
    /// Reads the node at the top of the stack, and if it is an object that has a member with
    /// the given name, adds the member to the current segment's output nodelist.
    SelectChildByName { name: Name },
    /// Reads the node at the top of the stack, and if it is an array that has an element at the
    /// given index, adds the element to the current segment's output nodelist.
    SelectElementAtIndex { index: Index },
    /// Reads the node at the top of the stack, and if it is an array, adds its elements specified
    /// by the given slice expression to the current segment's output nodelist.
    SelectSlice { slice: Slice },
    /// Pops a logical value from the stack, reads the node at the top of the stack,
    /// and if the value is true and the node is not the null node, adds it to the current segment's
    /// output nodelist.
    SelectNodeConditionally,
    /// Executes the given instructions in a loop until the stack is empty.
    WhileStackNotEmpty { instructions: Vec<Instruction> },
    /// Executes the given instructions in a loop for each child (element/member) of the node at
    /// the top of the stack.
    /// During the iteration, the current node is available via `PushCurrentFilterNode` instruction.
    FilterIteration { instructions: Vec<Instruction> },
    /// Pushes the current node of the filter iteration onto the stack.
    PushCurrentFilterNode,
    /// Executes the given subquery in a separate execution context and pushes the `true` value if
    /// the resultant nodelist is non-empty, otherwise pushes the `false` value.
    /// The input nodelist of the first segment of the subquery consists of the root node or the
    /// current node of the filter iteration, depending on the value of `absolute`.
    ExistenceTest { absolute: bool, subquery: Query },
    /// Pops two values from the stack, applies the given comparison operator to them and pushes
    /// the resultant logical value onto the stack.
    Compare { op: ComparisonOp },
    /// Pops two logical values from the stack and pushes their conjunction onto the stack.
    And,
    /// Pops two logical values from the stack and pushes their disjunction onto the stack.
    Or,
    /// Pops a logical value from the stack and pushes its negation onto the stack.
    Not
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