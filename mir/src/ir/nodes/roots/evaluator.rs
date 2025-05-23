use crate::ir::{BackLink, Builder, Link, Node, Op, Owner, Parent, Root, Singleton};
use miden_diagnostics::{SourceSpan, Spanned};

/// A MIR Root to represent a Evaluator definition
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Root)]
pub struct Evaluator {
    // Parameters of the evaluator.
    // each parameter Identifier in the ast corresponds to a Vec<Parameter>
    pub parameters: Vec<Vec<Link<Op>>>,
    // Operations contained in the Evaluator
    pub body: Link<Vec<Link<Op>>>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
}

impl Evaluator {
    pub fn create(
        parameters: Vec<Vec<Link<Op>>>,
        body: Vec<Link<Op>>,
        span: SourceSpan,
    ) -> Link<Root> {
        let res = Link::new(Root::Evaluator(Self {
            parameters,
            body: Link::new(body),
            span,
            ..Default::default()
        }));
        let node = Node::Evaluator(BackLink::from(res.clone()));
        res.as_evaluator_mut().unwrap()._node = Singleton::from(node);
        let owner = Owner::Evaluator(BackLink::from(res.clone()));
        res.as_evaluator_mut().unwrap()._owner = Singleton::from(owner);
        res
    }
}

impl Parent for Evaluator {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}
