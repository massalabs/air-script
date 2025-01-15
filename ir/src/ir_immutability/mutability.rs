use super::{BackLink, Link};
use std::ops::{Deref, DerefMut};
#[test]
fn test_mutability_final_design() {
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
            self.as_node().update(&other.as_node());
            self.update(other);
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
    match add.borrow().deref() {
        Op::Call(call) => println!("add.args: {:?}", call.args),
        _ => panic!("expected Call, found {:?}", add),
    };
    match node_add.borrow().deref() {
        Node::Call(op) => match op.to_link().unwrap().borrow().deref() {
            Op::Call(call) => println!("node_add.args: {:?}", call.args),
            _ => panic!("expected Call, found {:?}", op),
        },
        _ => panic!("expected Call, found {:?}", node_add),
    };
    // std::mem::swap(add.borrow(), &mut call.borrow());
    // -------------- ^^^^^^^^^^^^ expected mutable reference `&mut Ref<'_, test_current::Call>`
    //                         found struct `Ref<'_, test_current::Add>`

    //*node_add.borrow_mut() = *node_call.try_as_call().unwrap().borrow()
    // expected Node, found Call
}
