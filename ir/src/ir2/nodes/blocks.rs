use crate::ir2::{IsNode, Node};

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Function {
    #[node(args, ret, body)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Evaluator {
    #[node(expr)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct If {
    #[node(cond, then_branch, else_branch)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct For {
    #[node(iterators, body, selector)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Fold {
    #[node(iterator, operator, initial_value)]
    node: Node,
}
