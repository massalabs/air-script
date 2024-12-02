use crate::ir2::{IsNode, Node};

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Add {
    #[node(lhs, rhs)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Sub {
    #[node(lhs, rhs)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Mul {
    #[node(lhs, rhs)]
    node: Node,
}
