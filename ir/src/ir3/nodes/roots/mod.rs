mod evaluator;
mod function;

pub use evaluator::Evaluator;
pub use function::Function;

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Parameter {
    position: usize,
}

impl Parameter {
    pub fn new(position: usize) -> Self {
        Self { position }
    }
}
