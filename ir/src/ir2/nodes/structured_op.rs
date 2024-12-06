use crate::{ir2::{IsNode, Node}, FoldOperator};

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
