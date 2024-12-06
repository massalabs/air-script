mod constant_propagation;
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
pub use self::visitor_old::{Graph, VisitOld, VisitContextOld, VisitOrderOld};

//mod inlining;
mod translate;
mod unrolling;
mod visitor;
//pub use self::inlining::Inlining;
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
