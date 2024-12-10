use crate::ir3::{Evaluator, Function};

/// The root nodes of the MIR Graph
/// These represent the top level functions and evaluators
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Root {
    Function(Function),
    Evaluator(Evaluator),
    #[default]
    None,
}
