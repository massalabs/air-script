mod constant_propagation;
mod inlining;
mod translate_old;
mod translate;
mod value_numbering;
mod visitor;
mod visitor_old;
mod unrolling;

pub use self::constant_propagation::ConstantPropagation;
pub use self::inlining::Inlining;
pub use self::translate_old::AstToMirOld;
pub use self::translate::AstToMir;
pub use self::value_numbering::ValueNumbering;
pub use self::visitor_old::{Graph, VisitOld, VisitContextOld, VisitOrderOld};
pub use self::visitor::{Visit, VisitContext, VisitOrder};
pub use self::unrolling::Unrolling;

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
