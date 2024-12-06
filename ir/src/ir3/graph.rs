use crate::ir3::{Link, Root};

#[derive(Debug, Default)]
pub struct Graph {
    roots: Vec<Link<Root>>,
}

impl Graph {
    pub fn new(roots: Vec<Link<Root>>) -> Self {
        Self { roots }
    }
}
