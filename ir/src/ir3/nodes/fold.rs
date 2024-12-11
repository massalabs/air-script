use crate::ir3::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Fold {
    pub parent: BackLink<Owner>,
    pub iterator: Link<Op>,
    pub operator: FoldOperator,
    pub initial_value: Link<Op>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FoldOperator {
    Add,
    Mul,
    #[default]
    None,
}

impl Fold {
    pub fn new(iterator: Link<Op>, operator: FoldOperator, initial_value: Link<Op>) -> Self {
        Self {
            iterator,
            operator,
            initial_value,
            ..Default::default()
        }
    }
    pub fn as_op(self) -> Op {
        Op::Fold(self)
    }
    pub fn as_owner(self) -> Owner {
        Owner::Fold(self)
    }
    pub fn as_node(self) -> Node {
        Node::Fold(self)
    }
}

impl Link<Fold> {
    pub fn as_op(self) -> Link<Op> {
        Link::new(Op::Fold(self.borrow().clone()))
    }
    pub fn as_owner(self) -> Link<Owner> {
        Link::new(Owner::Fold(self.borrow().clone()))
    }
    pub fn as_node(self) -> Link<Node> {
        Link::new(Node::Fold(self.borrow().clone()))
    }
}

impl Parent for Fold {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.iterator.clone(), self.initial_value.clone()])
    }
}

impl Child for Fold {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct FoldBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    iterator: Option<Link<Op>>,
    operator: Option<FoldOperator>,
    initial_value: Option<Link<Op>>,
}

type FoldBuilderEmpty = FoldBuilder<(BackLink<Owner>, NotSet, NotSet, NotSet)>;
type FoldBuilderA = FoldBuilder<(BackLink<Owner>, Link<Op>, NotSet, NotSet)>;
type FoldBuilderB = FoldBuilder<(BackLink<Owner>, NotSet, FoldOperator, NotSet)>;
type FoldBuilderC = FoldBuilder<(BackLink<Owner>, NotSet, NotSet, Link<Op>)>;
type FoldBuilderAB = FoldBuilder<(BackLink<Owner>, Link<Op>, FoldOperator, NotSet)>;
type FoldBuilderAC = FoldBuilder<(BackLink<Owner>, Link<Op>, NotSet, Link<Op>)>;
type FoldBuilderBC = FoldBuilder<(BackLink<Owner>, NotSet, FoldOperator, Link<Op>)>;
type FoldBuilderFull = FoldBuilder<(BackLink<Owner>, Link<Op>, FoldOperator, Link<Op>)>;

impl Builder for Fold {
    type BuilderEmpty = FoldBuilderEmpty;
    type BuilderFull = FoldBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        FoldBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parent: self.parent,
            iterator: Some(self.iterator),
            operator: Some(self.operator),
            initial_value: Some(self.initial_value),
        }
    }
}

impl Default for FoldBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            iterator: None,
            operator: None,
            initial_value: None,
        }
    }
}

impl FoldBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> FoldBuilderA {
        self.iterator = Some(iterator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn operator(mut self, operator: FoldOperator) -> FoldBuilderB {
        self.operator = Some(operator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> FoldBuilderC {
        self.initial_value = Some(initial_value);
        unsafe { std::mem::transmute(self) }
    }
}

impl FoldBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> Self {
        self.iterator = Some(iterator);
        self
    }
    pub fn operator(mut self, operator: FoldOperator) -> FoldBuilderAB {
        self.operator = Some(operator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> FoldBuilderAC {
        self.initial_value = Some(initial_value);
        unsafe { std::mem::transmute(self) }
    }
}

impl FoldBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> FoldBuilderAB {
        self.iterator = Some(iterator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn operator(mut self, operator: FoldOperator) -> Self {
        self.operator = Some(operator);
        self
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> FoldBuilderBC {
        self.initial_value = Some(initial_value);
        unsafe { std::mem::transmute(self) }
    }
}

impl FoldBuilderC {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> FoldBuilderAC {
        self.iterator = Some(iterator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn operator(mut self, operator: FoldOperator) -> FoldBuilderBC {
        self.operator = Some(operator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> Self {
        self.initial_value = Some(initial_value);
        self
    }
}

impl FoldBuilderAB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> Self {
        self.iterator = Some(iterator);
        self
    }
    pub fn operator(mut self, operator: FoldOperator) -> Self {
        self.operator = Some(operator);
        self
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> FoldBuilderFull {
        self.initial_value = Some(initial_value);
        unsafe { std::mem::transmute(self) }
    }
}

impl FoldBuilderAC {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> Self {
        self.iterator = Some(iterator);
        self
    }
    pub fn operator(mut self, operator: FoldOperator) -> FoldBuilderFull {
        self.operator = Some(operator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> Self {
        self.initial_value = Some(initial_value);
        self
    }
}

impl FoldBuilderBC {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> FoldBuilderFull {
        self.iterator = Some(iterator);
        unsafe { std::mem::transmute(self) }
    }
    pub fn operator(mut self, operator: FoldOperator) -> Self {
        self.operator = Some(operator);
        self
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> Self {
        self.initial_value = Some(initial_value);
        self
    }
}

impl FoldBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterator(mut self, iterator: Link<Op>) -> Self {
        self.iterator = Some(iterator);
        self
    }
    pub fn operator(mut self, operator: FoldOperator) -> Self {
        self.operator = Some(operator);
        self
    }
    pub fn initial_value(mut self, initial_value: Link<Op>) -> Self {
        self.initial_value = Some(initial_value);
        self
    }
    pub fn build(self) -> Fold {
        Fold {
            parent: self.parent,
            iterator: self.iterator.unwrap(),
            operator: self.operator.unwrap(),
            initial_value: self.initial_value.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ir3::{Add, Evaluator, Mul};

    use super::*;

    #[test]
    fn test_fold_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default()));
        let fold = Fold::builder()
            .parent(parent.clone())
            .iterator(Link::new(Op::Add(Add::default())))
            .operator(FoldOperator::Add)
            .initial_value(Link::new(Op::Mul(Mul::default())))
            .build();
        assert_eq!(
            fold,
            Fold {
                parent: parent.clone().into(),
                iterator: Link::new(Op::Add(Add::default())),
                operator: FoldOperator::Add,
                initial_value: Link::new(Op::Mul(Mul::default())),
            }
        );
    }
}
