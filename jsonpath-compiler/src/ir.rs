use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

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
pub enum Segment {
    ChildSegment { selectors: Vec<Selector> },
    DescendantSegment { selectors: Vec<Selector> },
}

impl Segment {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        let mut segment_selectors: &Vec<Selector> = &vec![];
        if let Segment::ChildSegment { selectors } = self {
            write!(f, "ChildSegment {{\n")?;
            segment_selectors = selectors;
        }
        if let Segment::DescendantSegment { selectors } = self {
            write!(f, "DescendantSegment {{\n")?;
            segment_selectors = selectors;
        }
        for instruction in segment_selectors {
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
pub enum Selector {
    /// Selects all children (elements/members) of the node.
    AllChildren,
    /// If the node is an object that has a member with the given name, selects the member.
    ChildByName { name: Name },
    /// If the node is an array that has an element at the given index, selects the element.
    ElementAtIndex { index: Index },
    /// If the node is an array, selects its elements specified by the given slice expression.
    Slice { slice: Slice },
    // Selects all children satisfying the given filter expressoin.
    Filter { logical_expression: LogicalExpression },
}

impl Selector {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            Selector::AllChildren => write!(f, "SelectAllChildren"),
            Selector::ChildByName { name } => write!(f, "SelectChildByName({name})"),
            Selector::ElementAtIndex { index } => write!(f, "SelectElementAtIndex({index})"),
            Selector::Slice { slice } => write!(f, "SelectSlice({slice})"),
            Selector::Filter { logical_expression } => {
                write!(f, "Filter(\n")?;
                logical_expression.fmt(f, indent + 1)?;
                write!(f, "\n")?;
                write_indent(f, indent)?;
                write!(f, ")")
            }
        }
    }
}

impl Display for Selector {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum SingularSelector {
    /// If the node is an object that has a member with the given name, selects the member.
    ChildByName { name: Name },
    /// If the node is an array that has an element at the given index, selects the element.
    ElementAtIndex { index: Index },
}

impl SingularSelector {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            SingularSelector::ChildByName { name } => write!(f, "SelectChildByName({name})"),
            SingularSelector::ElementAtIndex { index } => write!(f, "SelectElementAtIndex({index})"),
        }
    }
}

impl Display for SingularSelector {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum LogicalExpression {
    Or { lhs: Box<LogicalExpression>, rhs: Box<LogicalExpression> },
    And { lhs: Box<LogicalExpression>, rhs: Box<LogicalExpression> },
    Not { expr: Box<LogicalExpression> },
    Comparison { lhs: Comparable, rhs: Comparable, op: ComparisonOp },
    ExistenceTest { absolute: bool, subquery: Query },
}

impl LogicalExpression {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            LogicalExpression::Or { lhs, rhs } => {
                LogicalExpression::fmt_binop(f, indent, "Or", (*lhs).deref(), (*rhs).deref())
            }
            LogicalExpression::And { lhs, rhs } => {
                LogicalExpression::fmt_binop(f, indent, "And", (*lhs).deref(), (*rhs).deref())
            }
            LogicalExpression::Not { expr } => {
                write!(f, "Not(\n")?;
                (*expr).deref().fmt(f, indent + 1)?;
                write!(f, "\n")?;
                write_indent(f, indent)?;
                write!(f, ")")
            }
            LogicalExpression::Comparison { lhs, rhs, op } => {
                write!(f, "{op}(\n")?;
                lhs.fmt(f, indent + 1)?;
                write!(f, ",\n")?;
                rhs.fmt(f, indent + 1)?;
                write!(f, "\n")?;
                write_indent(f, indent)?;
                write!(f, ")")
            }
            LogicalExpression::ExistenceTest { absolute, subquery } => {
                write!(f, "{}ExistenceTest(\n", if *absolute { "Absolute" } else { "Relative" })?;
                subquery.fmt(f, indent + 1)?;
                write_indent(f, indent)?;
                write!(f, ")")
            }
        }
    }

    fn fmt_binop(f: &mut Formatter, indent: u16, op: &str, lhs: &LogicalExpression, rhs: &LogicalExpression) -> fmt::Result {
        write!(f, "{op}(\n")?;
        lhs.fmt(f, indent + 1)?;
        write!(f, ",\n")?;
        rhs.fmt(f, indent + 1)?;
        write!(f, "\n")?;
        write_indent(f, indent)?;
        write!(f, ")")
    }
}

impl Display for LogicalExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum Comparable {
    SingularQuery { absolute: bool, selectors: Vec<SingularSelector> },
    Literal { literal: Literal },
}

impl Comparable {
    fn fmt(&self, f: &mut Formatter, indent: u16) -> fmt::Result {
        write_indent(f, indent)?;
        match self {
            Comparable::SingularQuery { absolute, selectors } => {
                write!(f, "{}SingularQuery(", if *absolute { "Absolute" } else { "Relative" })?;
                if selectors.len() > 0 {
                    write!(f, "\n")?;
                    for selector in selectors {
                        selector.fmt(f, indent + 1)?;
                        write!(f, "\n")?;
                    }
                    write_indent(f, indent)?;
                }
                write!(f, ")")

            }
            Comparable::Literal { literal } => write!(f, "{}", literal),
        }
    }
}

impl Display for Comparable {
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