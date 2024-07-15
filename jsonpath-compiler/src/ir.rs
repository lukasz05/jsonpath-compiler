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
    PushRootNode,

    PopAndPushAllChildren,
    PopAndPushChildByName { name: Name },
    PopAndPushElementAtIndex { index: Index },

    SelectNode,
    SelectAllChildren,
    SelectChildByName { name: Name },
    SelectElementAtIndex { index: Index },
    SelectSlice { slice: Slice },

    WhileStackNotEmpty { instructions: Vec<Instruction> },
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