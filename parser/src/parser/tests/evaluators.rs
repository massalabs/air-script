use miden_diagnostics::{SourceSpan, Span};

use crate::ast::*;

use super::ParseTest;

// EVALUATOR FUNCTIONS
// ================================================================================================

#[test]
fn ev_fn_main_cols() {
    let source = "
    mod test

    ev advance_clock([clk: int]) {
        enf clk' = clk + 1;
    }";

    let mut expected = Module::new(ModuleType::Library, SourceSpan::UNKNOWN, ident!(test));
    expected.evaluators.insert(
        ident!(advance_clock),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(advance_clock),
            vec![trace_segment!(0, "%0", [(ScalarType::Int, clk, 1)])],
            vec![enforce!(eq!(access!(clk, 1), add!(access!(clk), int!(1))))],
        ),
    );
    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn ev_fn_call_simple() {
    let source = "
    def test

    trace_columns {
        main: [clk: int],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        enf advance_clock([clk]);
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected
        .trace_columns
        .push(trace_segment!(0, "$main", [(ScalarType::Int, clk, 1)]));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new_vector(SourceSpan::UNKNOWN, ident!(inputs), 2),
    );
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(bounded_access!(a, Boundary::First), int!(0)))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(call!(advance_clock(vector!(access!(clk)))))],
    ));

    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn ev_fn_call() {
    let source = "
    def test

    trace_columns {
        main: [a: int[2], b: int[4], c: int[6]],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        enf advance_clock([a, b[1..3], c[2..4]]);
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [
            (ScalarType::Int, a, 2),
            (ScalarType::Int, b, 4),
            (ScalarType::Int, c, 6)
        ]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new_vector(SourceSpan::UNKNOWN, ident!(inputs), 2),
    );
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(bounded_access!(a, Boundary::First), int!(0)))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(call!(advance_clock(vector!(
            access!(a),
            slice!(b, 1..3),
            slice!(c, 2..4)
        ))))],
    ));

    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn ev_fn_call_inside_ev_fn() {
    let source = "
    mod test

    ev ev_func([clk: int]) {
        enf advance_clock([clk]);
    }";

    let mut expected = Module::new(ModuleType::Library, SourceSpan::UNKNOWN, ident!(test));
    let body = vec![enforce!(call!(advance_clock(vector!(access!(clk)))))];
    expected.evaluators.insert(
        ident!(ev_func),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(ev_func),
            vec![trace_segment!(0, "%0", [(ScalarType::Int, clk, 1)])],
            body,
        ),
    );

    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn ev_fn_call_with_more_than_two_args() {
    let source = "
    def test

    trace_columns {
        main: [a: int, b: int, c: int],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        enf advance_clock([a], [b], [c]);
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [
            (ScalarType::Int, a, 1),
            (ScalarType::Int, b, 1),
            (ScalarType::Int, c, 1)
        ]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new_vector(SourceSpan::UNKNOWN, ident!(inputs), 2),
    );
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(bounded_access!(a, Boundary::First), int!(0)))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(call!(advance_clock(
            vector!(access!(a)),
            vector!(access!(b)),
            vector!(access!(c))
        )))],
    ));

    ParseTest::new().expect_module_ast(source, expected);
}

// INVALID USE OF EVALUATOR FUNCTIONS
// ================================================================================================

#[test]
fn ev_fn_def_with_empty_final_arg() {
    let source = "
    mod test

    ev ev_func([clk], []) {
        enf clk' = clk + 1
    }";
    ParseTest::new().expect_module_diagnostic(source, "the last trace segment cannot be empty");
}

#[test]
fn ev_fn_call_with_no_args() {
    let source = "
    def test

    trace_columns {
        main: [clk],
    }

    integrity_constraints {
        enf advance_clock()
    }";
    ParseTest::new().expect_unrecognized_token(source);
}

#[test]
fn ev_fn_with_invalid_params() {
    let source = "
    mod test
    ev advance_clock() {
        enf clk' = clk + 1
    }";
    ParseTest::new().expect_unrecognized_token(source);

    let source = "
    mod test
    ev advance_clock([clk] [a, b]) {
        enf clk' = clk + 1
    }";
    ParseTest::new().expect_unrecognized_token(source);

    let source = "
    mod test
    ev advance_clock(, [a, b]) {
        enf clk' = clk + 1
    }";
    ParseTest::new().expect_unrecognized_token(source);

    let source = "
    mod test
    ev advance_clock([clk],) {
        enf clk' = clk + 1
    }";
    ParseTest::new().expect_unrecognized_token(source);
}
