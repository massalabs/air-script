/*mod constant_propagation;
mod value_numbering;

pub use self::constant_propagation::ConstantPropagation;
pub use self::value_numbering::ValueNumbering;

mod inlining_old;
mod translate_old;
mod unrolling_old;
mod visitor_old;
pub use self::inlining_old::InliningOld;
pub use self::translate_old::AstToMirOld;
pub use self::unrolling_old::UnrollingOld;
pub use self::visitor_old::{Graph, VisitContextOld, VisitOld, VisitOrderOld};*/

mod inlining;
mod translate;
mod unrolling;
mod visitor;
pub use self::inlining::Inlining;
pub use self::translate::AstToMir;
pub use self::unrolling::Unrolling;
pub use self::visitor::{Visit, VisitContext, VisitOrder};

use std::collections::HashMap;

use air_pass::Pass;

use crate::ir3::{
    Add, Boundary, Call, Enf, Fold, For, Graph, If, IsParent, LeafNode, Link, Matrix, MiddleNode,
    Mul, NodeType, RootNode, Scope, Sub, Vector,
};

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

pub fn duplicate_node(node: Link<NodeType>) -> Link<NodeType> {
    match node.borrow().deref() {
        NodeType::RootNode(RootNode::Graph(graph)) => {
            let mut new_graph = Graph::default();
            let children = graph.get_children().borrow().deref().clone();
            let new_children = children
                .iter()
                .map(|child| duplicate_node(*child))
                .collect();
            for new_child in new_children {
                new_graph.add_child(new_child);
            }
            return new_graph.into();
        }
        NodeType::LeafNode(leaf_node) => match leaf_node {
            LeafNode::Value(value_leaf) => {
                return value_leaf.data.clone().into();
            }
            LeafNode::Parameter(parameter_leaf) => {
                return parameter_leaf.data.clone().into();
            }
        },
        NodeType::MiddleNode(middle_node) => match middle_node {
            MiddleNode::Call(call) => {
                let arguments = call.arguments();
                let function = call.function();
                let new_arguments = arguments.iter().map(|arg| duplicate_node(arg)).collect();
                return Call::new(function, new_arguments).into();
            }
            MiddleNode::Function(_function) => {
                unreachable!();
            }
            MiddleNode::Evaluator(_evaluator) => {
                unreachable!();
            }
            MiddleNode::Add(add) => {
                let lhs = add.lhs();
                let rhs = add.rhs();
                let new_lhs_node = duplicate_node(lhs);
                let new_rhs_node = duplicate_node(rhs);
                return Add::new(new_lhs_node, new_rhs_node).into();
            }
            MiddleNode::Sub(sub) => {
                let lhs = sub.lhs();
                let rhs = sub.rhs();
                let new_lhs_node = duplicate_node(lhs);
                let new_rhs_node = duplicate_node(rhs);
                return Sub::new(new_lhs_node, new_rhs_node).into();
            }
            MiddleNode::Mul(mul) => {
                let lhs = mul.lhs();
                let rhs = mul.rhs();
                let new_lhs_node = duplicate_node(lhs);
                let new_rhs_node = duplicate_node(rhs);
                return Mul::new(new_lhs_node, new_rhs_node).into();
            }
            MiddleNode::Scope(scope) => {
                let new_scope = Scope::default();
                let children = scope.get_children().borrow().deref().clone();
                for child in children {
                    let new_child = duplicate_node(child);
                    new_scope.add_child(new_child);
                }
                return new_scope.into();
            }
            MiddleNode::If(if_node) => {
                let cond = if_node.cond();
                let then_branch = if_node.then_branch();
                let else_branch = if_node.else_branch();
                let new_cond = duplicate_node(cond);
                let new_then_branch = duplicate_node(then_branch);
                let new_else_branch = duplicate_node(else_branch);
                return If::new(new_cond, new_then_branch, new_else_branch).into();
            }
            MiddleNode::For(for_node) => {
                let iterators = for_node.iterators();
                let body = for_node.body();
                let selector = for_node.selector();
                let new_iterators = iterators
                    .iter()
                    .map(|iterator| duplicate_node(iterator))
                    .collect();
                let new_body = duplicate_node(body);
                let new_selector = selector.map(|selector| duplicate_node(selector));
                return For::new(new_iterators, new_body, new_selector).into();
            }
            MiddleNode::Fold(fold) => {
                let iterator = fold.iterator();
                let operator = fold.operator;
                let initial_value = fold.initial_value();
                let new_iterator = duplicate_node(iterator);
                let new_initial_value = duplicate_node(initial_value);
                return Fold::new(new_iterator, operator, new_initial_value).into();
            }
            MiddleNode::Boundary(boundary) => {
                let expr = boundary.expr();
                let kind = boundary.kind;
                let new_expr = duplicate_node(expr);
                return Boundary::new(new_expr, kind).into();
            }
            MiddleNode::Accessor(access) => {
                let indexable = access.indexable();
                let access_type = access.access_type;
                let new_indexable = duplicate_node(indexable);
                return Accessor::new(new_indexable, access_type).into();
            }
            MiddleNode::Enf(enf) => {
                let expr = enf.expr();
                let new_expr = duplicate_node(expr);
                return Enf::new(expr).into();
            }
            MiddleNode::Vector(vector) => {
                let children = vector.get_children().borrow().deref();
                let new_children = children
                    .iter()
                    .map(|child| duplicate_node(*child))
                    .collect();
                return Vector::new(new_children).into();
            }
            MiddleNode::Matrix(matrix) => {
                let new_matrix = Vec::new();
                let children = matrix.get_children().borrow().deref();
                for row in children.iter() {
                    let row_children = row.get_children().borrow().deref();
                    let new_row = row_children
                        .iter()
                        .map(|child| duplicate_node(*child))
                        .collect();
                    new_matrix.push(new_row);
                }
                return Matrix::new(new_matrix).into();
            }
        },
    }
}

// Helper used to duplicate nodes and their children recursively, used during Inlining and Unrolling
// Additionnally, if a Leaf is a Parameter, it is replaced with the corresponding item of the replace_parameter_list Vec.
// This is useful for inlining function calls (and replacing their parameters with the arguments of the call) for Inlining,
// and for unrolling loops (and replacing their parameters with the iterator values) for Unrolling.
// Inlining: replace_parameter_list = arguments should be the arguments from the Call()
// Unrolling: replace_parameter_list = self.for_inlining_context.unwrap().iterators
pub fn duplicate_node_or_replace(
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
                MiddleNode::Accessor(access) => {
                    let expr_node = access.indexable();
                    let new_expr_node = current_replace_map.get(&expr_node).unwrap().clone();
                    let new_node = Accessor::new(new_expr_node, access.access_type).into();
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
                MiddleNode::For(for_node) => {
                    let iterators = for_node.iterators();
                    let body = for_node.body();
                    let selector = for_node.selector();
                    let new_iterators = iterators
                        .iter()
                        .map(|node| current_replace_map.get(node).unwrap().clone())
                        .collect();
                    let new_body = current_replace_map.get(&body).unwrap().clone();
                    let new_selector =
                        selector.map(|node| current_replace_map.get(node).unwrap().clone());
                    let new_node = For::new(new_iterators, new_body, new_selector).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Enf(enf) => {
                    let expr = enf.expr();
                    let new_expr = current_replace_map.get(&expr).unwrap().clone();
                    let new_node = Enf::new(new_expr).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Call(call) => {
                    // Note: the function is not replaced
                    let function = call.function();
                    let arguments = call.arguments();
                    let new_arguments = arguments
                        .iter()
                        .map(|node| current_replace_map.get(node).unwrap().clone())
                        .collect();
                    let new_node = Call::new(function, new_arguments).into();
                    current_replace_map.insert(node, new_node);
                }
                MiddleNode::Function(_function) => {}
                MiddleNode::Evaluator(_evaluator) => {}
            }
        }
    }
}
