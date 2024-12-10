use crate::ir3::{Parameter, Value};

/// The Final nodes of the MIR Graph.
/// Currently unused in the structure but will be used in the next visitor pattern implementation.
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Leaf {
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
}
