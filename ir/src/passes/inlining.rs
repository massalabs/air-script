use std::ops::ControlFlow;

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{MirGraph, NodeIndex, Operation};

//pub struct Inlining<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

pub struct Inlining {}

//impl<'p> Pass for Inlining<'p> {}
impl Pass for Inlining {
    type Input<'a> = MirGraph;
    type Output<'a> = MirGraph;
    type Error = ();

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        match self.run_visitor(&mut ir) {
            ControlFlow::Continue(()) => Ok(ir),
            ControlFlow::Break(err) => Err(err),
        }
    }
}

// impl<'a> Inlining<'a> {
//     pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
//         Self { diagnostics }
//         Self {}
//     }
// }
impl Inlining {
    pub fn new() -> Self {
        Self {}
    }
    //TODO MIR: Implement inlining pass on MIR
    // 1. Understand the basics of the previous inlining process
    // 2. Remove what is done during lowering from AST to MIR (unroll, ...)
    // 3. Check how it translates to the MIR structure
    fn run_visitor(&mut self, ir: &mut MirGraph) -> ControlFlow<()> {
        let last_node_index = NodeIndex(ir.num_nodes() - 1);
        let last_node = ir.node(&last_node_index);
        println!("Last node: {:?}", last_node);
        self.inline_rec(ir, last_node_index);
        ControlFlow::Continue(())
    }

    fn inline_rec(&mut self, ir: &mut MirGraph, def_node_index: NodeIndex) {
        let def_node = ir.node(&def_node_index);
        println!("Def node: {:?}", def_node);
        let body_node_indexes = match &def_node.op {
            Operation::Definition(args, ret, body) => body.clone(),
            _ => return,
        };
        println!("Body node indexes: {:?}", body_node_indexes);
        for op_idx in body_node_indexes {
            println!("op_idx = {:?}", op_idx);
            let op = ir.node(&op_idx).op.clone();
            if let Operation::Call(func_idx, arg_idxs) = op {
                println!("Call: {:?} {:?}", func_idx, arg_idxs);
                let func_node = ir.node(&func_idx);
                if let Operation::Definition(func_args, func_ret, func_body) = &func_node.op {
                    println!(
                        "args: {:?} ret: {:?} body: {:?}",
                        func_args, func_ret, func_body
                    );
                    ir.replace_by_chunk(op_idx.clone(), func_body.clone());
                };
            };
        }
    }
}
