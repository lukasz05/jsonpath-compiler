pub mod generator;

#[derive(Debug)]
pub struct Query {
    segments: Vec<Segment>
}

#[derive(Debug)]
pub struct Segment {
    instructions: Vec<Instruction>
}

#[derive(Debug)]
pub enum Instruction {
    GetChildByName {node: NodeParam, name: Name},
    GetElementAtIndex {node: NodeParam, index: Index},

    SelectNode {node: NodeParam},
    SelectAllChildren {node: NodeParam },
    SelectSlice {node: NodeParam, slice: Slice},

    WhileStackNotEmpty {instructions: Vec<Instruction>},
    PushAllChildren {node: NodeParam },
    PushNode {node: NodeParam},
    PopNode,
}

#[derive(Debug)]
pub enum NodeParam {
    RootNode,
    CurrentNode,
    VarNode
}

#[derive(Debug)]
pub struct Name(String);

#[derive(Debug)]
pub struct Index(i64);

#[derive(Debug)]
pub struct Slice {start: i64, end: Option<i64>, step: i64}