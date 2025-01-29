use crate::ir::{Add, Builder, ConstantValue, Link, MirValue, Mul, Op, SpannedMirValue, Value};

fn value(val: u64) -> Link<Op> {
    Value::create(SpannedMirValue {
        value: MirValue::Constant(ConstantValue::Felt(val)),
        ..Default::default()
    })
}

fn build_a() -> Link<Op> {
    Add::builder().lhs(value(1)).rhs(value(2)).build()
}

fn build_b() -> Link<Op> {
    Mul::builder().lhs(value(3)).rhs(value(1)).build()
}

#[test]
fn test_ir_mutability() {
    // no existing Node
    eprintln!("===== no existing Node =====");
    let a = build_a();
    eprintln!("a = {}", a.debug());
    let b = build_b();
    eprintln!("b = {}", b.debug());
    a.set(&b);

    eprintln!("============================");
    // a with existing Node
    eprintln!("===== a existing Node =====");
    let a_with_node = build_a();
    let a_node = a_with_node.as_node();
    eprintln!("a_node = {}", a_node.debug());
    eprintln!("a_with_node = {}", a_with_node.debug());
    let b = build_b();
    a_with_node.set(&b);
    eprintln!("a_node = {}", a_node.debug());
    eprintln!("a_with_node = {}", a_with_node.debug());
    eprintln!("============================");

}
