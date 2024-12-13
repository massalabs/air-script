use std::ops::Deref;

use crate::ir::{ForChild, Link, Node, Op, Parent};

pub enum VisitOrder {
    Manual,
    DepthFirst,
    PostOrder,
}
pub trait VisitDefault {}
pub trait VisitContext {
    type Graph;
    fn visit(&mut self, graph: &mut Self::Graph, link: Link<Node>);
    fn as_stack_mut(&mut self) -> &mut Vec<Link<Node>>;
    fn boundary_roots(&self, graph: &Self::Graph) -> Link<Vec<Link<Node>>>;
    fn integrity_roots(&self, graph: &Self::Graph) -> Link<Vec<Link<Node>>>;
    fn visit_order(&self) -> VisitOrder;
}
pub trait Visit: VisitContext {
    fn run(&mut self, graph: &mut Self::Graph) {
        match self.visit_order() {
            VisitOrder::Manual => self.visit_manual(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
        }
        while let Some(node_index) = self.next_node() {
            self.visit(graph, node_index);
        }
    }
    fn visit_manual(&mut self, graph: &mut Self::Graph) {
        for root_index in self
            .boundary_roots(graph)
            .borrow()
            .iter()
            .chain(self.integrity_roots(graph).borrow().iter())
        {
            self.visit(graph, root_index.clone());
        }
    }
    fn visit_postorder(&mut self, graph: &mut Self::Graph) {
        for root_index in self
            .boundary_roots(graph)
            .borrow()
            .iter()
            .chain(self.integrity_roots(graph).borrow().iter())
        {
            self.visit_later(root_index.clone());
            let mut last: Option<Link<Node>> = None;
            while let Some(link) = self.peek() {
                let children = get_children(link.clone());
                if children.borrow().is_empty()
                    || last.is_some()
                        && (last.clone().unwrap().as_op().is_some()
                            && children
                                .borrow()
                                .contains(&last.clone().unwrap().as_op().unwrap()))
                {
                    self.visit(graph, link.clone());
                    self.next_node();
                    last = Some(link.clone());
                } else {
                    for child in children.borrow().iter().rev() {
                        self.visit_later(child.clone().as_node());
                    }
                }
            }
        }
    }
    fn visit_depthfirst(&mut self, graph: &mut Self::Graph) {
        for root_index in self
            .boundary_roots(graph)
            .borrow()
            .iter()
            .chain(self.integrity_roots(graph).borrow().iter())
        {
            self.visit_later(root_index.clone());
            while let Some(link) = self.next_node() {
                let children = get_children(link.clone());
                for child in children.borrow().iter().rev() {
                    self.visit_later(child.clone().as_node());
                }
                self.visit(graph, link);
            }
        }
    }
    fn peek(&mut self) -> Option<Link<Node>> {
        self.as_stack_mut().last().cloned()
    }
    fn next_node(&mut self) -> Option<Link<Node>> {
        self.as_stack_mut().pop()
    }
    fn visit_later(&mut self, link: Link<Node>) {
        self.as_stack_mut().push(link);
    }
}

impl<T> Visit for T where T: VisitContext + VisitDefault {}

fn get_children(link: Link<Node>) -> Link<Vec<Link<Op>>> {
    match link.borrow().deref() {
        Node::Function(function) => function.children(),
        Node::Evaluator(evaluator) => evaluator.children(),
        Node::Enf(enf) => enf.children(),
        Node::Boundary(boundary) => boundary.children(),
        Node::Add(add) => add.children(),
        Node::Sub(sub) => sub.children(),
        Node::Mul(mul) => mul.children(),
        Node::If(if_node) => if_node.children(),
        Node::For(for_node) => {
            let mut op_children = Vec::new();
            let for_children = for_node.children();
            for for_child in for_children.borrow().iter() {
                match for_child.borrow().deref() {
                    ForChild::Iterators(link) => {
                        for vector in link.borrow().iter() {
                            op_children.push(vector.clone());
                        }
                    }
                    ForChild::Expr(link) => op_children.push(link.borrow().clone().into()),
                    ForChild::Selector(link) => op_children.push(link.borrow().clone().into()),
                };
            }
            Link::new(op_children)
        }
        Node::Call(call) => call.children(),
        Node::Fold(fold) => fold.children(),
        Node::Vector(vector) => vector.children(),
        Node::Matrix(matrix) => matrix
            .children()
            .borrow()
            .deref()
            .clone()
            .iter()
            .map(|v| v.clone().as_op().into())
            .collect::<Vec<_>>()
            .into(),
        Node::Accessor(accessor) => accessor.children(),
        Node::Parameter(_parameter) => Link::new(vec![]),
        Node::Value(_value) => Link::new(vec![]),
        Node::None => Link::new(vec![]),
    }
}
