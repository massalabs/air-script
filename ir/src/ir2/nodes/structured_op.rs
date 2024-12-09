use crate::ir2::{IsNode, Node};


#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum FoldOperator {
    #[default]
    Add,
    Mul,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Fold {
    pub operator: FoldOperator,
    #[node(iterator, initial_value)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Call {
    pub arguments_count: usize,
    #[node(function, arguments)]
    node: Node,
}
