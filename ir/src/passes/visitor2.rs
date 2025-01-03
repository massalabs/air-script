use std::ops::Deref;

use crate::ir::{
    Accessor, Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, Graph, If, Link, Matrix,
    Mul, Node, Parameter, Parent, Sub, Value, Vector,
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
                self.scan_node(_graph, child.clone().as_node());
            }
        }
    }
    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) {
        match node.borrow().deref() {
            Node::Function(f) => self.visit_function(graph, f.clone()),
            Node::Evaluator(e) => self.visit_evaluator(graph, e.clone()),
            Node::Enf(e) => self.visit_enf(graph, e.clone()),
            Node::Boundary(b) => self.visit_boundary(graph, b.clone()),
            Node::Add(a) => self.visit_add(graph, a.clone()),
            Node::Sub(s) => self.visit_sub(graph, s.clone()),
            Node::Mul(m) => self.visit_mul(graph, m.clone()),
            Node::If(i) => self.visit_if(graph, i.clone()),
            Node::For(f) => self.visit_for(graph, f.clone()),
            Node::Call(c) => self.visit_call(graph, c.clone()),
            Node::Fold(f) => self.visit_fold(graph, f.clone()),
            Node::Vector(v) => self.visit_vector(graph, v.clone()),
            Node::Matrix(m) => self.visit_matrix(graph, m.clone()),
            Node::Accessor(a) => self.visit_accessor(graph, a.clone()),
            Node::Parameter(p) => self.visit_parameter(graph, p.clone()),
            Node::Value(v) => self.visit_value(graph, v.clone()),
            Node::None => {}
        }
    }
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
    fn visit_parameter(&mut self, _graph: &mut Graph, _parameter: Link<Parameter>) {}
    fn visit_value(&mut self, _graph: &mut Graph, _value: Link<Value>) {}
}
