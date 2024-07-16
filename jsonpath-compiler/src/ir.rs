pub mod generator;

#[derive(Debug)]
pub struct Query {
    segments: Vec<Segment>,
}

#[derive(Debug)]
pub struct Segment {
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    PushLiteral { literal: Literal },
    PushRootNode,
    PushAllChildren,

    Pop,
    PopAndPushAllChildren,
    PopAndPushChildByName { name: Name },
    PopAndPushElementAtIndex { index: Index },

    Duplicate,

    SelectAllChildren,
    SelectChildByName { name: Name },
    SelectElementAtIndex { index: Index },
    SelectSlice { slice: Slice },
    SelectNodeCond,

    WhileStackNotEmpty { instructions: Vec<Instruction> },

    ExistenceTest { absolute: bool, subquery: Query },
    Compare { op: ComparisonOp },
    And,
    Or,
    Not,
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
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

#[derive(Debug)]
pub struct Name(String);

#[derive(Debug)]
pub struct Index(i64);

#[derive(Debug)]
pub struct Slice {
    start: i64,
    end: Option<i64>,
    step: i64,
}