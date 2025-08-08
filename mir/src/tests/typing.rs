use miden_diagnostics::Span;
use pretty_assertions::assert_eq;

use crate::ir::{
    Builder, Enf, Link, MirValue, Mul, SpannedMirValue, Sub, TraceAccess, Value,
    extract_integrity_roots, strip_spans,
};

use super::compile;

#[test]
fn test_typing() {
    let code = "
    def test

    trace_columns {
        main: [a, b],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        let b2 = assert_bool(b);
        let c = select(a, b2);
        enf c = 42;
    }
    fn select(x: felt, selector: bool) -> felt {
        return x * selector;
    }
    ";
    let mir = compile(code)
        .unwrap_or_else(|_| panic!("Failed to compile, see diagnostics for more information"));
    let integrity_constraints = extract_integrity_roots(mir.constraint_graph())
        .iter()
        .map(|n| n.as_op().expect("Expected integrity constraint to be an Op"))
        .collect::<Vec<_>>();
    dbg!(&integrity_constraints);
    let expected = vec![
        Enf::builder()
            .span(Default::default())
            .expr(
                Sub::builder()
                    .span(Default::default())
                    .lhs(
                        Mul::builder()
                            .span(Default::default())
                            .lhs(
                                Value::builder()
                                    .value(SpannedMirValue {
                                        value: MirValue::TraceAccess(TraceAccess {
                                            segment: 0,
                                            column: 0,
                                            row_offset: 0,
                                        }),
                                        ..Default::default()
                                    })
                                    .build(),
                            )
                            .rhs(todo!())
                            .build(),
                    )
                    .rhs(todo!())
                    .build(),
            )
            .build(),
    ];
    assert_eq!(integrity_constraints, expected);
}
