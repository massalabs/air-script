use std::ops::Deref;

use crate::ir::{
    Accessor, Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, Graph, If, Leaf, Link,
    Matrix, Mul, Node, Owner, Parent, Sub, Vector,
};
pub trait Visitor {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>>;
    fn run(&mut self, graph: &mut Graph) {
        eprintln!("Running visitor");
        eprintln!("Function nodes: {:?}", graph.get_function_nodes());
        for function in graph.get_function_nodes() {
            self.visit_node(graph, function.clone().as_node());
        }
        eprintln!("Evaluator nodes: {:?}", graph.get_evaluator_nodes());
        for evaluator in graph.get_evaluator_nodes() {
            self.visit_node(graph, evaluator.clone().as_node());
        }
        println!("Work stack: {:?}", self.work_stack());
        panic!("Not implemented");
        while let Some(node) = self.work_stack().pop() {
            self.visit_node(graph, node);
        }
    }
    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) {
        eprintln!("Visiting node {:?}", node);
        if let Some(owner) = node.clone().as_owner() {
            self.visit_owner(graph, owner.clone());
        } else if let Some(op) = node.as_leaf() {
            self.visit_leaf(graph, op.clone());
        } else {
            panic!("Unknown node type");
        }
    }
    fn visit_children(&mut self, graph: &mut Graph, node: Link<Owner>) {
        eprintln!("Visiting children of {:?}", node);
        for child in node.children().borrow().iter() {
            eprintln!("Visiting child {:?}", child);
            self.work_stack().push(child.clone().as_node());
        }
    }
    fn visit_owner(&mut self, graph: &mut Graph, owner: Link<Owner>) {
        eprintln!("Visiting owner {:?}", owner);
        self.work_stack().push(owner.clone().as_node());
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
        self.visit_children(graph, owner.clone());
    }
    fn visit_leaf(&mut self, graph: &mut Graph, op: Link<Leaf>) {
        eprintln!("Visiting leaf {:?}", op);
        self.work_stack().push(op.clone().as_node());
    }
    fn visit_function(&mut self, graph: &mut Graph, function: Link<Function>) {}
    fn visit_evaluator(&mut self, graph: &mut Graph, evaluator: Link<Evaluator>) {}
    fn visit_enf(&mut self, graph: &mut Graph, enf: Link<Enf>) {}
    fn visit_boundary(&mut self, graph: &mut Graph, boundary: Link<Boundary>) {}
    fn visit_add(&mut self, graph: &mut Graph, add: Link<Add>) {}
    fn visit_sub(&mut self, graph: &mut Graph, sub: Link<Sub>) {}
    fn visit_mul(&mut self, graph: &mut Graph, mul: Link<Mul>) {}
    fn visit_if(&mut self, graph: &mut Graph, if_node: Link<If>) {}
    fn visit_for(&mut self, graph: &mut Graph, for_node: Link<For>) {}
    fn visit_call(&mut self, graph: &mut Graph, call: Link<Call>) {}
    fn visit_fold(&mut self, graph: &mut Graph, fold: Link<Fold>) {}
    fn visit_vector(&mut self, graph: &mut Graph, vector: Link<Vector>) {}
    fn visit_matrix(&mut self, graph: &mut Graph, matrix: Link<Matrix>) {}
    fn visit_accessor(&mut self, graph: &mut Graph, accessor: Link<Accessor>) {}
}
