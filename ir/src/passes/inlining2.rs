use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{
    ir::{Graph, Link, Mir, Node},
    CompileError,
};

use super::visitor2::Visitor;

pub struct Inlining {
    // general context
    work_stack: Vec<Link<Node>>,
}
impl Inlining {
    pub fn new() -> Self {
        Self { work_stack: vec![] }
    }
}

impl Pass for Inlining {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut pass = Inlining::new();
        Visitor::run(&mut pass, ir.constraint_graph_mut());
        Ok(ir)
    }
}

impl Visitor for Inlining {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn visit_function(&mut self, graph: &mut Graph, function: Link<crate::ir::Function>) {
        eprintln!("Visiting function {:?}", function);
    }
}
