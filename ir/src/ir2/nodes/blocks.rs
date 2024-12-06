use crate::ir2::{IsNode, Node};
#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Function {
    pub args_count: usize,
    #[node(args, ret, body)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Evaluator {
    pub args_count: usize,
    #[node(args, body)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct If {
    #[node(cond, then_branch, else_branch)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct For {
    pub iterators_count: usize,
    #[node(iterators, body, selector)]
    node: Node,
}
