use crate::ir2::{IsNode, Node};
use air_parser::ast::Boundary as BoundaryKind;

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Enf {
    #[node(expr)]
    node: Node,
}

#[derive(Clone, Eq, PartialEq, IsNode)]
pub struct Boundary {
    kind: BoundaryKind,
    #[node(expr)]
    node: Node,
}
