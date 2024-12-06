use crate::ir3::{BackLink, Child, Link, Op, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct If {
    parent: BackLink<Op>,
    condition: Link<Op>,
    then_branch: Link<Op>,
    else_branch: Link<Op>,
}

impl If {
    pub fn new(condition: Link<Op>, then_branch: Link<Op>, else_branch: Link<Op>) -> Self {
        Self {
            condition,
            then_branch,
            else_branch,
            ..Default::default()
        }
    }
}

impl Parent for If {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![
            self.condition.clone(),
            self.then_branch.clone(),
            self.else_branch.clone(),
        ])
    }
}

impl Child for If {
    type Parent = Op;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
