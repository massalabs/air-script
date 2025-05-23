use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent, Singleton};
use miden_diagnostics::{SourceSpan, Spanned};

/// A MIR operation to represent the exponentiation of a MIR op, `lhs` by another, `rhs`
///
/// Note: `rhs` should be a constant integer after all the passes
///
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Exp {
    pub parents: Vec<BackLink<Owner>>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
}

impl Exp {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let res = Link::new(Op::Exp(Self {
            lhs,
            rhs,
            span,
            ..Default::default()
        }));
        let node = Node::Exp(BackLink::from(res.clone()));
        res.as_exp_mut().unwrap()._node = Singleton::from(node);
        let owner = Owner::Exp(BackLink::from(res.clone()));
        res.as_exp_mut().unwrap()._owner = Singleton::from(owner);
        res
    }
}

impl Parent for Exp {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Exp {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.parents.clone()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.push(parent.into());
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.retain(|p| *p != parent.clone().into());
    }
}
