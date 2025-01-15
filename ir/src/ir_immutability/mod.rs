mod link;
use std::cell::Ref;
use std::mem::swap;
use std::ops::{Deref, DerefMut};

use link::{BackLink, Link};

#[test]
fn test_mutability_current() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Add {
        lhs: Link<Op>,
        rhs: Link<Op>,
    }
    impl Add {
        fn create(lhs: Link<Op>, rhs: Link<Op>) -> Link<Add> {
            Link::new(Add { lhs, rhs })
        }
    }
    impl Link<Add> {
        fn as_node(&self) -> Link<Node> {
            Link::new(Node::Add(self.clone()))
        }
        fn as_op(&self) -> Link<Op> {
            Link::new(Op::Add(self.clone()))
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Call {
        callee: Link<Op>,
        args: Vec<Link<Op>>,
    }
    impl Call {
        fn create(callee: Link<Op>, args: Vec<Link<Op>>) -> Link<Call> {
            Link::new(Call { callee, args })
        }
    }
    impl Link<Call> {
        fn as_node(&self) -> Link<Node> {
            Link::new(Node::Call(self.clone()))
        }
        fn as_op(&self) -> Link<Op> {
            Link::new(Op::Call(self.clone()))
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Value {
        value: i32,
    }
    impl Value {
        fn create(value: i32) -> Link<Value> {
            Link::new(Value { value })
        }
    }
    impl Link<Value> {
        fn as_node(&self) -> Link<Node> {
            Link::new(Node::Value(self.clone()))
        }
        fn as_op(&self) -> Link<Op> {
            Link::new(Op::Value(self.clone()))
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Op {
        Add(Link<Add>),
        Call(Link<Call>),
        Value(Link<Value>),
    }
    impl Link<Op> {
        fn as_node(&self) -> Link<Node> {
            match self.borrow().deref() {
                Op::Add(add) => add.as_node(),
                Op::Call(call) => call.as_node(),
                Op::Value(value) => value.as_node(),
            }
        }
        fn as_add(&self) -> Option<Link<Add>> {
            match self.borrow().deref() {
                Op::Add(add) => Some(add.clone()),
                _ => None,
            }
        }
        fn as_call(&self) -> Option<Link<Call>> {
            match self.borrow().deref() {
                Op::Call(call) => Some(call.clone()),
                _ => None,
            }
        }
        fn as_value(&self) -> Option<Link<Value>> {
            match self.borrow().deref() {
                Op::Value(value) => Some(value.clone()),
                _ => None,
            }
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Node {
        Add(Link<Add>),
        Call(Link<Call>),
        Value(Link<Value>),
    }
    impl Link<Node> {
        fn as_op(&self) -> Link<Op> {
            match self.borrow().deref() {
                Node::Add(add) => add.as_op(),
                Node::Call(call) => call.as_op(),
                Node::Value(value) => value.as_op(),
            }
        }
        fn as_add(&self) -> Option<Link<Add>> {
            match self.borrow().deref() {
                Node::Add(add) => Some(add.clone()),
                _ => None,
            }
        }
        fn as_call(&self) -> Option<Link<Call>> {
            match self.borrow().deref() {
                Node::Call(call) => Some(call.clone()),
                _ => None,
            }
        }
        fn as_value(&self) -> Option<Link<Value>> {
            match self.borrow().deref() {
                Node::Value(value) => Some(value.clone()),
                _ => None,
            }
        }
    }

    let a = Value::create(1);
    let b = Value::create(2);
    let add = Add::create(a.as_op(), b.as_op());
    dbg!(&add);
    let node_add = add.as_node();
    dbg!(&node_add);
    let add2 = node_add.as_add().unwrap();
    dbg!(&add);
    dbg!(&add2);
    dbg!(std::ptr::eq(
        add.borrow().deref() as *const Add,
        add2.borrow().deref() as *const Add
    ));
    dbg!(&add);
    let other = Add::create(Value::create(7).as_op(), Value::create(8).as_op());
    dbg!(&other);
    eprintln!("swapping");
    add.swap(&other);
    dbg!(&add);
    dbg!(&other);
    dbg!(&add2);
    let expected_add = Add::create(Value::create(7).as_op(), Value::create(8).as_op());
    let expected_other = Add::create(Value::create(1).as_op(), Value::create(2).as_op());
    assert_eq!(add, expected_add);
    assert_eq!(add2, expected_add);
    assert_eq!(other, expected_other);
    assert_eq!(node_add, expected_add.as_node());
    eprintln!("swapped successfully");
    let add3 = Add::create(Value::create(9).as_op(), Value::create(10).as_op());
    let add4 = Add::create(Value::create(11).as_op(), Value::create(12).as_op());
    let node_add3 = add3.as_node();
    let node_add4 = add4.as_node();
    dbg!(&add3);
    dbg!(&add4);
    eprintln!("setting add3 to add4");
    *add3.borrow_mut().deref_mut() = add4.borrow().deref().clone();
    dbg!(&add3);
    dbg!(&add4);
    assert_eq!(add3, add4);
    assert_eq!(node_add3, node_add4);
    eprintln!("set add3 to add4 successfully");

    // it's currently not possible to swap the an add for a call
    // because the types are different
    let c = Value::create(3);
    let d = Value::create(4);
    let callee = Add::create(c.as_op(), d.as_op());
    let e = Value::create(5);
    let f = Value::create(6);
    let args = vec![e.as_op(), f.as_op()];
    let call = Call::create(callee.as_op(), args);
    dbg!(&call);
    let node_call = call.as_node();
    dbg!(&node_call);
    // std::mem::swap(add.borrow(), &mut call.borrow());
    // -------------- ^^^^^^^^^^^^ expected mutable reference `&mut Ref<'_, test_current::Call>`
    //                         found struct `Ref<'_, test_current::Add>`

    //*node_add.borrow_mut() = *node_call.as_call().unwrap().borrow()
    // expected Node, found Call
}

#[test]
fn test_mutability_wrap_enum() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Add {
        lhs: Link<Op>,
        rhs: Link<Op>,
    }
    impl Add {
        fn create(lhs: Link<Op>, rhs: Link<Op>) -> Link<Op> {
            Op::Add(Add { lhs, rhs }).into()
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Call {
        callee: Link<Root>,
        args: Vec<Link<Op>>,
    }
    impl Call {
        fn create(callee: Link<Root>, args: Vec<Link<Op>>) -> Link<Op> {
            Op::Call(Call { callee, args }).into()
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Value {
        value: i32,
    }
    impl Value {
        fn create(value: i32) -> Link<Op> {
            Op::Value(Value { value }).into()
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Function {
        params: Vec<Link<Op>>,
        body: Link<Op>,
    }
    impl Function {
        fn create(params: Vec<Link<Op>>, body: Link<Op>) -> Link<Root> {
            Root::Function(Function { params, body }.into()).into()
        }
    }

    // We wrap every Op in a Link<Op> to make it possible to swap nodes of different types
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Op {
        Add(Add),
        Call(Call),
        Value(Value),
    }
    impl Link<Op> {
        pub fn as_node(&self) -> Link<Node> {
            Node::Op(self.clone()).into()
        }
        pub fn try_as_add(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Op::Add(_) => Some(self.clone()),
                _ => None,
            }
        }
        pub fn try_as_call(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Op::Call(_) => Some(self.clone()),
                _ => None,
            }
        }
        pub fn try_as_value(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Op::Value(_) => Some(self.clone()),
                _ => None,
            }
        }
        pub fn set(&self, other: &Link<Op>) {
            match other.borrow().deref() {
                Op::Add(add) => {
                    *self.borrow_mut().deref_mut() = Op::Add(add.clone());
                }
                Op::Call(call) => {
                    *self.borrow_mut().deref_mut() = Op::Call(call.clone());
                }
                Op::Value(value) => {
                    *self.borrow_mut().deref_mut() = Op::Value(value.clone());
                }
            }
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Root {
        Function(Link<Function>),
    }
    impl Link<Root> {
        pub fn as_node(&self) -> Link<Node> {
            Node::Root(self.clone()).into()
        }
        pub fn as_function(&self) -> Option<Link<Function>> {
            match self.borrow().deref() {
                Root::Function(function) => Some(function.clone()),
            }
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Node {
        Op(Link<Op>),
        Root(Link<Root>),
    }
    impl Link<Node> {
        pub fn as_op(&self) -> Link<Op> {
            match self.borrow().deref() {
                Node::Op(op) => op.clone(),
                _ => panic!("expected Op, found {:?}", self),
            }
        }
        pub fn as_root(&self) -> Link<Root> {
            match self.borrow().deref() {
                Node::Root(root) => root.clone(),
                _ => panic!("expected Root, found {:?}", self),
            }
        }
        pub fn try_as_add(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Node::Op(op) => op.try_as_add(),
                _ => None,
            }
        }
        pub fn try_as_call(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Node::Op(op) => op.try_as_call(),
                _ => None,
            }
        }
        pub fn try_as_value(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Node::Op(op) => op.try_as_value(),
                _ => None,
            }
        }
    }

    let a = Value::create(1);
    let b = Value::create(2);
    let add = Add::create(a, b);
    dbg!(&add);
    let node_add = add.as_node();
    dbg!(&node_add);
    let add2 = node_add.as_op();
    dbg!(&add);
    dbg!(&add2);
    let left_op_ptr = add.borrow().deref() as *const Op;
    let right_op_ptr = add2.borrow().deref() as *const Op;
    dbg!(std::ptr::eq(left_op_ptr, right_op_ptr));
    assert!(std::ptr::eq(left_op_ptr, right_op_ptr));
    dbg!(&add);
    let other = Add::create(Value::create(7), Value::create(8));
    dbg!(&other);
    eprintln!("swapping");
    add.set(&other);
    dbg!(&add);
    dbg!(&other);
    dbg!(&add2);
    let expected_add = Add::create(Value::create(7), Value::create(8));
    assert_eq!(add, expected_add);
    assert_eq!(add2, expected_add);
    assert_eq!(node_add, expected_add.as_node());
    eprintln!("swapped successfully");
    let add3 = Add::create(Value::create(9), Value::create(10));
    let add4 = Add::create(Value::create(11), Value::create(12));
    let node_add3 = add3.as_node();
    let node_add4 = add4.as_node();
    dbg!(&add3);
    dbg!(&add4);
    eprintln!("setting add3 to add4");
    *add3.borrow_mut().deref_mut() = add4.borrow().deref().clone();
    dbg!(&add3);
    dbg!(&add4);
    assert_eq!(add3, add4);
    assert_eq!(node_add3, node_add4);
    eprintln!("set add3 to add4 successfully");
    // let's try to swap an Add for a Call
    let add = Add::create(Value::create(1), Value::create(2));
    let node_add = add.as_node();
    dbg!(&add);
    let c = Value::create(3);
    let d = Value::create(4);
    let callee = Function::create(vec![c], d);
    let e = Value::create(5);
    let f = Value::create(6);
    let args = vec![e, f];
    let call = Call::create(callee, args);
    dbg!(&call);
    let node_call = call.as_node();
    dbg!(&node_call);
    eprintln!("\n");
    dbg!(&add);
    eprintln!("swapping and casting");
    add.set(&call);
    dbg!(&add);
    dbg!(&call);
    dbg!(&node_add);
    dbg!(&node_call);
    let expected_call = Call::create(
        Function::create(vec![Value::create(3)], Value::create(4)),
        vec![Value::create(5), Value::create(6)],
    );
    assert_eq!(node_add.as_op(), expected_call);
    assert_eq!(node_call.as_op(), expected_call);
    // assertion `left == right` failed
    //
    // std::mem::swap(add.borrow(), &mut call.borrow());
    // -------------- ^^^^^^^^^^^^ expected mutable reference `&mut Ref<'_, test_current::Call>`
    //                         found struct `Ref<'_, test_current::Add>`

    //*node_add.borrow_mut() = *node_call.try_as_call().unwrap().borrow()
    // expected Node, found Call
}

#[test]
fn test_mutability_wrap_op_singleton() {
    #[derive(Clone, Debug, Eq)]
    struct Add {
        lhs: Link<Op>,
        rhs: Link<Op>,
        _node: Option<Link<Node>>,
    }
    impl PartialEq for Add {
        fn eq(&self, other: &Self) -> bool {
            self.lhs == other.lhs && self.rhs == other.rhs
        }
    }
    impl Add {
        fn create(lhs: Link<Op>, rhs: Link<Op>) -> Link<Op> {
            Op::Add(Add {
                lhs,
                rhs,
                _node: None,
            })
            .into()
        }
    }
    #[derive(Clone, Debug, Eq)]
    struct Call {
        callee: Link<Root>,
        args: Vec<Link<Op>>,
        _node: Option<Link<Node>>,
    }
    impl PartialEq for Call {
        fn eq(&self, other: &Self) -> bool {
            self.callee == other.callee && self.args == other.args
        }
    }
    impl Call {
        fn create(callee: Link<Root>, args: Vec<Link<Op>>) -> Link<Op> {
            Op::Call(Call {
                callee,
                args,
                _node: None,
            })
            .into()
        }
    }
    #[derive(Clone, Debug, Eq)]
    struct Value {
        value: i32,
        _node: Option<Link<Node>>,
    }
    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            self.value == other.value
        }
    }
    impl Value {
        fn create(value: i32) -> Link<Op> {
            Op::Value(Value { value, _node: None }).into()
        }
    }
    #[derive(Clone, Debug, Eq)]
    struct Function {
        params: Vec<Link<Op>>,
        body: Link<Op>,
        _node: Option<Link<Node>>,
    }
    impl PartialEq for Function {
        fn eq(&self, other: &Self) -> bool {
            self.params == other.params && self.body == other.body
        }
    }
    impl Function {
        fn create(params: Vec<Link<Op>>, body: Link<Op>) -> Link<Root> {
            Root::Function(Function {
                params,
                body,
                _node: None,
            })
            .into()
        }
    }

    // We wrap every Op in a Link<Op> to make it possible to swap nodes of different types
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Op {
        Add(Add),
        Call(Call),
        Value(Value),
    }
    impl Link<Op> {
        pub fn as_node(&self) -> Link<Node> {
            match self.borrow_mut().deref_mut() {
                Op::Add(Add {
                    _node: Some(node), ..
                }) => {
                    eprintln!(" -> returning existing node: {:?}", node);
                    node.clone()
                }
                Op::Add(ref mut add) => {
                    let node: Link<Node> = Node::Add(self.clone().into()).into();
                    eprintln!(" -> creating new node: {:?}", node);
                    add._node = Some(node.clone());
                    node
                }
                Op::Call(Call {
                    _node: Some(node), ..
                }) => {
                    eprintln!(" -> returning existing node: {:?}", node);
                    node.clone()
                }
                Op::Call(ref mut call) => {
                    let node: Link<Node> = Node::Call(self.clone().into()).into();
                    eprintln!(" -> creating new node: {:?}", node);
                    call._node = Some(node.clone());
                    node
                }
                Op::Value(Value {
                    _node: Some(node), ..
                }) => {
                    eprintln!(" -> returning existing node: {:?}", node);
                    node.clone()
                }
                Op::Value(ref mut value) => {
                    let node: Link<Node> = Node::Value(self.clone().into()).into();
                    eprintln!(" -> creating new node: {:?}", node);
                    value._node = Some(node.clone());
                    node
                }
            }
        }
        pub fn try_as_add(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Op::Add(_) => Some(self.clone()),
                _ => None,
            }
        }
        pub fn try_as_call(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Op::Call(_) => Some(self.clone()),
                _ => None,
            }
        }
        pub fn try_as_value(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Op::Value(_) => Some(self.clone()),
                _ => None,
            }
        }
        pub fn set(&self, other: &Link<Op>) {
            eprintln!("setting {:?}\n     to {:?}", &self, &other);
            let old = self.get_singletons();
            eprintln!("old: {:?}", old);
            self.update(other);
            self.update_singletons(old);
        }
        fn get_singletons(&self) -> Vec<Link<Node>> {
            vec![self.as_node()]
        }
        fn update_singletons(&self, olds: Vec<Link<Node>>) {
            eprintln!("updating singletons");
            eprintln!("old: {:?}", olds);
            eprintln!(
                "self._node: {:?}",
                match self.borrow().deref() {
                    Op::Add(add) => add._node.as_ref(),
                    Op::Call(call) => call._node.as_ref(),
                    Op::Value(value) => value._node.as_ref(),
                }
            );
            let mut updates_op: Vec<(Link<Op>, Link<Op>)> = vec![];
            let mut updates_root: Vec<(Link<Root>, Link<Root>)> = vec![];
            for old in olds {
                match old.borrow().deref() {
                    Node::Add(old_add) => {
                        if let Some(old_node) = old_add.to_link().unwrap().try_as_add() {
                            let node = self.as_node().try_as_op().as_ref().unwrap().clone();
                            updates_op.push((old_node, node));
                        }
                    }
                    Node::Call(old_call) => {
                        if let Some(old_node) = old_call.to_link().unwrap().try_as_call() {
                            let node = self.as_node().try_as_op().as_ref().unwrap().clone();
                            updates_op.push((old_node, node));
                        }
                    }
                    Node::Value(old_value) => {
                        if let Some(old_node) = old_value.to_link().unwrap().try_as_value() {
                            let node = self.as_node().try_as_op().as_ref().unwrap().clone();
                            updates_op.push((old_node, node));
                        }
                    }
                    Node::Function(old_function) => {
                        if let Some(old_node) = old_function.to_link().unwrap().try_as_function() {
                            let node = self.as_node().try_as_root().as_ref().unwrap().clone();
                            updates_root.push((old_node, node));
                        }
                    }
                }
            }
            eprintln!("updates_op: {:#?}", updates_op);
            eprintln!("updates_root: {:#?}", updates_root);
            for (old, new) in updates_op {
                old.update(&new);
            }
            for (old, new) in updates_root {
                old.update(&new);
            }
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Root {
        Function(Function),
    }
    impl Link<Root> {
        pub fn as_node(&self) -> Link<Node> {
            match self.borrow_mut().deref_mut() {
                Root::Function(Function {
                    _node: Some(node), ..
                }) => node.clone(),
                Root::Function(ref mut function) => {
                    let node: Link<Node> = Node::Function(self.clone().into()).into();
                    function._node = Some(node.clone());
                    node
                }
            }
        }
        pub fn try_as_function(&self) -> Option<Link<Root>> {
            match self.borrow().deref() {
                Root::Function(_) => Some(self.clone()),
            }
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Node {
        Add(BackLink<Op>),
        Call(BackLink<Op>),
        Value(BackLink<Op>),
        Function(BackLink<Root>),
    }
    impl Link<Node> {
        pub fn try_as_op(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Node::Add(op) => op.to_link(),
                Node::Call(op) => op.to_link(),
                Node::Value(op) => op.to_link(),
                _ => None,
            }
        }
        pub fn try_as_root(&self) -> Option<Link<Root>> {
            match self.borrow().deref() {
                Node::Function(root) => root.to_link(),
                _ => None,
            }
        }
        pub fn try_as_add(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Node::Add(op) => op.to_link(),
                _ => None,
            }
        }
        pub fn try_as_call(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Node::Call(op) => op.to_link(),
                _ => None,
            }
        }
        pub fn try_as_value(&self) -> Option<Link<Op>> {
            match self.borrow().deref() {
                Node::Value(op) => op.to_link(),
                _ => None,
            }
        }
    }

    let a = Value::create(1);
    let b = Value::create(2);
    let mut add = Add::create(a, b);
    dbg!(&add);
    let node_add = add.as_node();
    dbg!(&node_add);
    let add2 = node_add.try_as_op().unwrap();
    dbg!(&add);
    dbg!(&add2);
    let left_op_ptr = add.borrow().deref() as *const Op;
    let right_op_ptr = add2.borrow().deref() as *const Op;
    dbg!(std::ptr::eq(left_op_ptr, right_op_ptr));
    assert!(std::ptr::eq(left_op_ptr, right_op_ptr));
    dbg!(&add);
    let other = Add::create(Value::create(7), Value::create(8));
    dbg!(&other);
    eprintln!("swapping");
    add.set(&other);
    dbg!(&add);
    dbg!(&other);
    dbg!(&add2);
    let expected_add = Add::create(Value::create(7), Value::create(8));
    assert_eq!(add, expected_add);
    assert_eq!(add2, expected_add);
    assert_eq!(node_add, expected_add.as_node());
    eprintln!("swapped successfully");
    let add3 = Add::create(Value::create(9), Value::create(10));
    let add4 = Add::create(Value::create(11), Value::create(12));
    let node_add3 = add3.as_node();
    let node_add4 = add4.as_node();
    dbg!(&add3);
    dbg!(&add4);
    eprintln!("setting add3 to add4");
    *add3.borrow_mut().deref_mut() = add4.borrow().deref().clone();
    dbg!(&add3);
    dbg!(&add4);
    assert_eq!(add3, add4);
    assert_eq!(node_add3, node_add4);
    eprintln!("set add3 to add4 successfully");
    // let's try to swap an Add for a Call
    let add = Add::create(Value::create(1), Value::create(2));
    let node_add = add.as_node();
    dbg!(&add);
    let c = Value::create(3);
    let d = Value::create(4);
    let callee = Function::create(vec![c], d);
    let e = Value::create(5);
    let f = Value::create(6);
    let args = vec![e, f];
    let call = Call::create(callee, args);
    dbg!(&call);
    let node_call = call.as_node();
    dbg!(&node_call);
    eprintln!("\n");
    dbg!(&add);
    eprintln!("swapping and casting");
    add.set(&call);
    dbg!(&add);
    dbg!(&call);
    dbg!(&node_add);
    dbg!(&node_call);
    let expected_call = Call::create(
        Function::create(vec![Value::create(3)], Value::create(4)),
        vec![Value::create(5), Value::create(6)],
    );
    assert_eq!(node_add.try_as_op().unwrap(), expected_call);
    assert_eq!(node_call.try_as_op().unwrap(), expected_call);
    // std::mem::swap(add.borrow(), &mut call.borrow());
    // -------------- ^^^^^^^^^^^^ expected mutable reference `&mut Ref<'_, test_current::Call>`
    //                         found struct `Ref<'_, test_current::Add>`

    //*node_add.borrow_mut() = *node_call.try_as_call().unwrap().borrow()
    // expected Node, found Call
}

#[test]
fn test_links() {
    let a = Link::new(1);
    let a1 = a.clone();
    let b = Link::new(2);
    let b1 = b.clone();
    dbg!(&a);
    dbg!(&a1);
    dbg!(&b);
    dbg!(&b1);
    a.swap(&b);
    dbg!(&a);
    dbg!(&a1);
    dbg!(&b);
    dbg!(&b1);
    assert_eq!(a, a1);
    assert_eq!(a, Link::new(2));
    assert_eq!(b, b1);
    assert_eq!(b, Link::new(1));
    assert!(std::ptr::eq(a.borrow().deref(), a1.borrow().deref()));
    assert!(std::ptr::eq(b.borrow().deref(), b1.borrow().deref()));

    let mut a = Link::new(1);
    let a1 = a.clone();
    let b = Link::new(2);
    let b1 = b.clone();
    dbg!(&a);
    dbg!(&a1);
    dbg!(&b);
    dbg!(&b1);
    // NOTE:
    // clone_from only updates the current Link and not its clones
    // a.clone_from(&b);
    a.update(&b);
    dbg!(&a);
    dbg!(&a1);
    dbg!(&b);
    dbg!(&b1);
    assert_eq!(a, a1);
    assert_eq!(a, Link::new(2));
    assert_eq!(b, b1);
    assert_eq!(b, Link::new(2));
    assert!(std::ptr::eq(a.borrow().deref(), a1.borrow().deref()));
    assert!(std::ptr::eq(b.borrow().deref(), b1.borrow().deref()));
}
