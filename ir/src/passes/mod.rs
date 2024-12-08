mod constant_propagation;
mod value_numbering;
use std::collections::HashMap;
use std::ops::Deref;

use crate::ir2::{
    Add, Boundary, Fold, If, IsParent, LeafNode, Link, Matrix, MiddleNode, Mul, NodeType, Scope, Sub, Vector
};

pub use self::constant_propagation::ConstantPropagation;
pub use self::value_numbering::ValueNumbering;

mod inlining_old;
mod translate_old;
mod unrolling_old;
mod visitor_old;
pub use self::inlining_old::InliningOld;
pub use self::translate_old::AstToMirOld;
pub use self::unrolling_old::UnrollingOld;
pub use self::visitor_old::{Graph, VisitContextOld, VisitOld, VisitOrderOld};

mod inlining;
mod translate;
mod unrolling;
mod visitor;
pub use self::inlining::Inlining;
pub use self::translate::AstToMir;
pub use self::unrolling::Unrolling;
pub use self::visitor::{Visit, VisitContext, VisitOrder};

use air_pass::Pass;

pub struct DumpAst;
impl Pass for DumpAst {
    type Input<'a> = air_parser::ast::Program;
    type Output<'a> = air_parser::ast::Program;
    type Error = air_parser::SemanticAnalysisError;

    fn run<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        println!("{}", &input);
        Ok(input)
    }
}

// Helper used to duplicate nodes and their children recursively, used during Inlining and Unrolling
// Additionnally, if a Leaf is a Parameter, it is replaced with the corresponding item of the replace_parameter_list Vec.
// This is useful for inlining function calls (and replacing their parameters with the arguments of the call) for Inlining,
// and for unrolling loops (and replacing their parameters with the iterator values) for Unrolling.
// Inlining: replace_parameter_list = arguments should be the arguments from the Call()
// Unrolling: replace_parameter_list = self.for_inlining_context.unwrap().iterators
fn duplicate_node_or_replace(
    current_replace_map: &mut HashMap<Link<NodeType>, Link<NodeType>>,
    node: Link<NodeType>,
    replace_parameter_list: Vec<Link<NodeType>>,
) {
    match node.borrow().deref() {
        NodeType::RootNode(root_node) => unreachable!(),
        NodeType::LeafNode(leaf_node) => {
            match leaf_node {
                LeafNode::Value(value_leaf) => {
                    let new_node = value_leaf.data.clone().into();
                    current_replace_map.insert(node, new_node);
                }
                LeafNode::Parameter(parameter_leaf) => {
                    let new_node = replace_parameter_list[parameter_leaf.data.argument_position];
                    current_replace_map.insert(node, new_node);
                    // /!\ FIXME This removes the child from the For ?
                    // Should not be a problem as the For node is removed
                }
            }
        }
        NodeType::MiddleNode(middle_node) => {
            match middle_node {
                MiddleNode::Add(add) => {
                    let lhs = add.lhs();
                    let rhs = add.rhs();
                    let new_lhs_node = current_replace_map.get(&lhs).unwrap().clone();
                    let new_rhs_node = current_replace_map.get(&rhs).unwrap().clone();
                    let new_node = Add::new(new_lhs_node, new_rhs_node).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Sub(sub) => {
                    let lhs = sub.lhs();
                    let rhs = sub.rhs();
                    let new_lhs_node = current_replace_map.get(&lhs).unwrap().clone();
                    let new_rhs_node = current_replace_map.get(&rhs).unwrap().clone();
                    let new_node = Sub::new(new_lhs_node, new_rhs_node).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Mul(mul) => {
                    let lhs = mul.lhs();
                    let rhs = mul.rhs();
                    let new_lhs_node = current_replace_map.get(&lhs).unwrap().clone();
                    let new_rhs_node = current_replace_map.get(&rhs).unwrap().clone();
                    let new_node = Mul::new(new_lhs_node, new_rhs_node).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Scope(_scope) => {
                    let new_scope = Scope::default();
                    let children = _scope.get_children().borrow().deref().clone();
                    for child in children {
                        let new_child = current_replace_map.get(&child).unwrap().clone();
                        new_scope.add_child(new_child);
                    }
                    current_replace_map.insert(node, new_scope);
                }
                MiddleNode::If(if_node) => {
                    let cond = if_node.cond();
                    let then_branch = if_node.then_branch();
                    let else_branch = if_node.else_branch();

                    let new_cond = current_replace_map.get(&cond).unwrap().clone();
                    let new_then = current_replace_map.get(&then_branch).unwrap().clone();
                    let new_else = current_replace_map.get(&else_branch).unwrap().clone();
                    let new_node = If::new(new_cond, new_then, new_else).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Fold(fold) => {
                    let iter = fold.iterator();
                    let fold_operator = fold.operator.clone();
                    let acc = fold.initial_value();
                    let new_iter = current_replace_map.get(&iter).unwrap().clone();
                    let new_acc = current_replace_map.get(&acc).unwrap().clone();
                    let new_node = Fold::new(new_iter, fold_operator, new_acc).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Boundary(boundary) => {
                    let expr_node = boundary.expr();
                    let new_expr_node = current_replace_map.get(&expr_node).unwrap().clone();
                    let new_node = Boundary::new(new_expr_node, boundary.kind).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Vector(_vector) => {
                    let v = _vector.get_children().borrow().deref().clone();
                    let new_v = v
                        .iter()
                        .map(|node| current_replace_map.get(node).unwrap().clone())
                        .collect();
                    let new_node = Vector::new(new_v).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Matrix(_matrix) => {
                    let m = _matrix.get_children().borrow().deref().clone();
                    let new_m = m
                        .iter()
                        .map(|row| {
                            let row = row.get_children().borrow().deref().clone();
                            row.iter()
                                .map(|node| current_replace_map.get(node).unwrap().clone())
                                .collect()
                        })
                        .collect();
                    let new_node = Matrix::new(new_m).into();
                    current_replace_map.insert(node, new_node);
                }
                // These should not exist / be accessible from roots after inlining and in For
                MiddleNode::For(_for) => todo!(), // needed for inlining
                MiddleNode::Enf(_enf) => todo!(), // needed for inlining
                MiddleNode::Call(_call) => todo!(), // needed for inlining ? Unsure
                MiddleNode::Function(_function) => unreachable!(),
                MiddleNode::Evaluator(_evaluator) => unreachable!(),
            }
        }
    }
}
