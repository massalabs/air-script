use crate::ir2::{IsNode, Node};

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Fold {
    #[node(iterator, operator, initial_value)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Call {
    #[node(function, arguments)]
    node: Node,
}
