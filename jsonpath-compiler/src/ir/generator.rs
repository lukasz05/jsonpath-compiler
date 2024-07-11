use crate::ir::{Index, Instruction, Name, Query, Segment, Slice};
use crate::ir::Instruction::{GetChildByName, GetElementAtIndex, PopNode, PushAllChildren, PushNode,
                             SelectAllChildren, SelectNode, SelectSlice, WhileStackNotEmpty};
use crate::ir::NodeParam::{CurrentNode, VarNode};

pub fn generate(query_syntax: &rsonpath_syntax::JsonPathQuery) -> Query {
    Query {
        segments: query_syntax.segments().into_iter()
            .map(|segment_syntax| generate_segment(segment_syntax))
            .collect()
    }
}

fn generate_segment(segment_syntax: &rsonpath_syntax::Segment) -> Segment {
    match segment_syntax {
        rsonpath_syntax::Segment::Child(selectors) => generate_child_segment(selectors),
        rsonpath_syntax::Segment::Descendant(selectors) => generate_descendant_segment(selectors),
    }
}

fn generate_child_segment(selectors: &rsonpath_syntax::Selectors) -> Segment {
    Segment { instructions: generate_selectors(selectors) }
}

fn generate_descendant_segment(selectors: &rsonpath_syntax::Selectors) -> Segment {
    let instructions: Vec<Instruction> = vec![
        PushNode { node: CurrentNode },
        WhileStackNotEmpty {
            instructions: vec![
                PopNode,
                PushAllChildren { node: VarNode },
            ].into_iter().chain(generate_selectors(selectors).into_iter()).collect()
        },
    ];
    Segment { instructions }
}

fn generate_selectors(selectors: &rsonpath_syntax::Selectors) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = vec![];
    for selector in selectors.iter() {
        instructions.extend(generate_selector(selector));
    }
    instructions
}

fn generate_selector(selector_syntax: &rsonpath_syntax::Selector) -> Vec<Instruction> {
    let instructions: Vec<Instruction>;
    match selector_syntax {
        rsonpath_syntax::Selector::Name(name_syntax) => {
            let name = Name(name_syntax.unquoted().to_owned());
            instructions = vec![
                GetChildByName { node: CurrentNode, name },
                SelectNode { node: VarNode },
            ]
        }
        rsonpath_syntax::Selector::Wildcard => instructions = vec![SelectAllChildren { node: CurrentNode }],
        rsonpath_syntax::Selector::Index(index_syntax) => {
            let index = generate_index(index_syntax);
            instructions = vec![
                GetElementAtIndex { node: CurrentNode, index },
                SelectNode { node: VarNode },
            ]
        }
        rsonpath_syntax::Selector::Slice(slice_syntax) => {
            let slice = generate_slice(slice_syntax);
            instructions = vec![SelectSlice { node: CurrentNode, slice: slice }]
        }
        rsonpath_syntax::Selector::Filter(_) => panic!("Filters not supported yet.")
    }
    instructions
}

fn generate_index(index_syntax: &rsonpath_syntax::Index) -> Index {
    Index(index_syntax_as_i64(index_syntax))
}

fn generate_slice(slice_syntax: &rsonpath_syntax::Slice) -> Slice {
    Slice {
        start: index_syntax_as_i64(&slice_syntax.start()),
        end: slice_syntax.end().map(|index_syntax| index_syntax_as_i64(&index_syntax)),
        step: step_syntax_as_i64(&slice_syntax.step())
    }
}

fn index_syntax_as_i64(index_syntax: &rsonpath_syntax::Index) -> i64 {
    match index_syntax {
        rsonpath_syntax::Index::FromStart(num) => num.as_u64() as i64,
        rsonpath_syntax::Index::FromEnd(num) => -(num.as_u64() as i64)
    }
}

fn step_syntax_as_i64(step_syntax: &rsonpath_syntax::Step) -> i64 {
    match step_syntax {
        rsonpath_syntax::Step::Forward(num) => num.as_u64() as i64,
        rsonpath_syntax::Step::Backward(num) => -(num.as_u64() as i64)
    }
}
