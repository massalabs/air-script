use std::ops::Deref;

use crate::ir::{
    Accessor, Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, Graph, If, Leaf, Link,
    Matrix, Mul, Node, Owner, Parent, Sub, Vector,
};

pub trait Visitor {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>>;
    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>>;
    fn run(&mut self, graph: &mut Graph) {
        for root in self.root_nodes_to_visit(graph) {
            self.scan_node(graph, root.clone());
        }
        while let Some(node) = self.work_stack().pop() {
            self.visit_node(graph, node);
        }
    }
    fn scan_node(&mut self, _graph: &Graph, node: Link<Node>) {
        self.work_stack().push(node.clone());
        if let Some(_owner) = node.clone().as_owner() {
            for child in node.children().borrow().iter() {
                self.work_stack().push(child.clone().as_node());
            }  
        }
    }
    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) {
        //eprintln!("Visiting node {:?}", node);
        if let Some(owner) = node.clone().as_owner() {
            self.visit_owner(graph, owner.clone());
        } else if let Some(op) = node.clone().as_leaf() {
            self.visit_leaf(graph, op.clone());
        }
    }
    fn visit_owner(&mut self, graph: &mut Graph, owner: Link<Owner>) {
        match owner.borrow().deref() {
            Owner::Function(_) => self.visit_function(graph, owner.clone().as_function().unwrap()),
            Owner::Evaluator(_) => {
                self.visit_evaluator(graph, owner.clone().as_evaluator().unwrap())
            }
            Owner::Enf(_) => self.visit_enf(graph, owner.clone().as_enf().unwrap()),
            Owner::Boundary(_) => self.visit_boundary(graph, owner.clone().as_boundary().unwrap()),
            Owner::Add(_) => self.visit_add(graph, owner.clone().as_add().unwrap()),
            Owner::Sub(_) => self.visit_sub(graph, owner.clone().as_sub().unwrap()),
            Owner::Mul(_) => self.visit_mul(graph, owner.clone().as_mul().unwrap()),
            Owner::If(_) => self.visit_if(graph, owner.clone().as_if().unwrap()),
            Owner::For(_) => self.visit_for(graph, owner.clone().as_for().unwrap()),
            Owner::Call(_) => self.visit_call(graph, owner.clone().as_call().unwrap()),
            Owner::Fold(_) => self.visit_fold(graph, owner.clone().as_fold().unwrap()),
            Owner::Vector(_) => self.visit_vector(graph, owner.clone().as_vector().unwrap()),
            Owner::Matrix(_) => self.visit_matrix(graph, owner.clone().as_matrix().unwrap()),
            Owner::Accessor(_) => self.visit_accessor(graph, owner.clone().as_accessor().unwrap()),
            Owner::None => {}
        }
    }
    fn visit_leaf(&mut self, _graph: &mut Graph, _op: Link<Leaf>) {}
    fn visit_function(&mut self, _graph: &mut Graph, _function: Link<Function>) {}
    fn visit_evaluator(&mut self, _graph: &mut Graph, _evaluator: Link<Evaluator>) {}
    fn visit_enf(&mut self, _graph: &mut Graph, _enf: Link<Enf>) {}
    fn visit_boundary(&mut self, _graph: &mut Graph, _boundary: Link<Boundary>) {}
    fn visit_add(&mut self, _graph: &mut Graph, _add: Link<Add>) {}
    fn visit_sub(&mut self, _graph: &mut Graph, _sub: Link<Sub>) {}
    fn visit_mul(&mut self, _graph: &mut Graph, _mul: Link<Mul>) {}
    fn visit_if(&mut self, _graph: &mut Graph, _if_node: Link<If>) {}
    fn visit_for(&mut self, _graph: &mut Graph, _for_node: Link<For>) {}
    fn visit_call(&mut self, _graph: &mut Graph, _call: Link<Call>) {}
    fn visit_fold(&mut self, _graph: &mut Graph, _fold: Link<Fold>) {}
    fn visit_vector(&mut self, _graph: &mut Graph, _vector: Link<Vector>) {}
    fn visit_matrix(&mut self, _graph: &mut Graph, _matrix: Link<Matrix>) {}
    fn visit_accessor(&mut self, _graph: &mut Graph, _accessor: Link<Accessor>) {}
}
