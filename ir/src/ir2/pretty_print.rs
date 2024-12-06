use std::{cell::RefCell, rc::Rc};
use super::{Graph, Link, NodeType};

#[derive(Debug, Clone)]
struct PrettyShared<'a> {
    pub var_count: usize,
    pub fn_count: usize,
    // BTreeMap from function index to function id
    pub fns: Vec<(Link<NodeType>, usize)>,
    pub roots: &'a [Link<NodeType>],
}

#[derive(Clone)]
struct PrettyCtx<'a> {
    pub graph: &'a Graph,
    pub indent: usize,
    pub nl: &'a str,
    pub in_block: bool,
    pub show_var_names: bool,
    pub shared: Rc<RefCell<PrettyShared<'a>>>,
}

impl<'a> PrettyCtx<'a> {
    fn new(graph: &'a Graph, roots: &'a [Link<NodeType>]) -> Self {
        let shared = Rc::new(RefCell::new(PrettyShared {
            var_count: 0,
            fn_count: 0,
            fns: Vec::new(),
            roots,
        }));
        Self {
            graph,
            indent: 0,
            nl: "\n",
            in_block: false,
            shared,
            show_var_names: true,
        }
    }

    fn add_indent(&self, indent: usize) -> Self {
        Self {
            indent: self.indent + indent,
            ..self.clone()
        }
    }

    fn with_indent(&self, indent: usize) -> Self {
        Self {
            indent,
            ..self.clone()
        }
    }

    fn increment_var_count(&self) -> Self {
        self.shared.borrow_mut().var_count += 1;
        self.clone()
    }

    fn increment_fn_count(&self, link: &Link<NodeType>) -> Self {
        let fn_count = self.shared.borrow().fn_count;
        self.shared.borrow_mut().fns.push((link.clone(), fn_count));
        self.shared.borrow_mut().fn_count += 1;
        self.clone()
    }

    fn with_nl(&self, nl: &'a str) -> Self {
        Self { nl, ..self.clone() }
    }

    fn with_in_block(&self, in_block: bool) -> Self {
        Self {
            in_block,
            ..self.clone()
        }
    }

    fn indent_str(&self) -> String {
        if self.nl == "\n" {
            "  ".repeat(self.indent)
        } else {
            "".to_string()
        }
    }

    fn show_var_names(&self, show_var_names: bool) -> Self {
        Self {
            show_var_names,
            ..self.clone()
        }
    }
}

impl std::fmt::Debug for PrettyCtx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrettyCtx")
            .field("indent", &self.indent)
            .field("nl", &self.nl)
            .field("in_block", &self.in_block)
            .field("var_count", &self.shared.borrow().var_count)
            .field("fn_count", &self.shared.borrow().fn_count)
            .finish()
    }
}

pub fn pretty(graph: &Graph, roots: &[Link<NodeType>]) -> String {
    let mut result = String::from("");
    let mut ctx = PrettyCtx::new(graph, roots);
    for root in roots {
        pretty_rec(root.clone(), &mut ctx, &mut result);
        ctx.shared.borrow_mut().var_count = 0; // reset var count for next function
    }
    result
}

#[allow(unused_variables)]
fn pretty_rec(link: Link<NodeType>, ctx: &mut PrettyCtx, result: &mut String) {
    
    todo!();

    /*match link.borrow().clone() {
        NodeType::RootNode(root_node) => {
            match root_node {
                super::RootNode::Graph(graph) => todo!(),
            }
        },
        NodeType::LeafNode(leaf_node) => {
            match leaf_node {
                super::LeafNode::Value(leaf) => todo!(),
                super::LeafNode::Parameter(leaf) => todo!(),
            }
            
        },
        NodeType::MiddleNode(middle_node) => {
            match middle_node {
                super::MiddleNode::Add(add) => todo!(),
                super::MiddleNode::Sub(sub) => todo!(),
                super::MiddleNode::Mul(mul) => todo!(),
                super::MiddleNode::Scope(scope) => todo!(),
                super::MiddleNode::Function(function) => {
                    let args = function.args();
                    let ret = function.ret();
                    let body = function.body();

                    result.push_str(&format!(
                        "{}fn f{}(",
                        ctx.indent_str(),
                        ctx.shared.borrow().fn_count
                    ));
                    ctx.increment_fn_count(&link.clone());
                    for (i, arg) in args_idx.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        pretty_rec(*arg, &mut ctx.with_indent(0).with_nl(""), result);
                    }
                    result.push_str(") -> ");
                    match ret_idx {
                        Some(ret_idx) => pretty_rec(
                            *ret_idx,
                            &mut ctx.with_indent(0).with_nl("").show_var_names(false),
                            result,
                        ),
                        None => result.push_str("()"),
                    }
                    result.push_str(" {\n");
                    for op_idx in body_idx {
                        pretty_rec(*op_idx, &mut ctx.add_indent(1).with_in_block(true), result);
                    }
                    result.push_str(&format!(
                        "{}return x{};\n",
                        ctx.add_indent(1).indent_str(),
                        ctx.shared.borrow().var_count
                    ));
                    result.push_str(&format!("{}}}", ctx.indent_str()));
                    if ctx.shared.borrow().fn_count != ctx.shared.borrow().roots.len() {
                        result.push_str("\n\n");
                    }
                },
                super::MiddleNode::Evaluator(evaluator) => todo!(),
                super::MiddleNode::If(_) => todo!(),
                super::MiddleNode::For(_) => todo!(),
                super::MiddleNode::Fold(fold) => todo!(),
                super::MiddleNode::Call(call) => todo!(),
                super::MiddleNode::Boundary(boundary) => todo!(),
                super::MiddleNode::Enf(enf) => todo!(),
                super::MiddleNode::Vector(vector) => todo!(),
                super::MiddleNode::Matrix(matrix) => todo!(),
            }
        },
    }*/


    /*match link.borrow() {

    }*/

    /*match op {
        Operation::Definition(args_idx, ret_idx, body_idx) => {
            
        }
        Operation::Value(spanned_val) => {
            let val = &spanned_val.value;
            match val {
                MirValue::Variable(ty, pos, _func) => {
                    if ctx.in_block {
                        result.push_str(&format!("x{}", pos));
                    } else {
                        if ctx.show_var_names {
                            result.push_str(&format!("{}x{}: ", ctx.indent_str(), pos));
                        };
                        result.push_str(&format!("{:?}{}", ty, ctx.nl));
                    }
                }
                MirValue::Constant(constant) => {
                    result.push_str(&format!("{}{:?}{}", ctx.indent_str(), constant, ctx.nl));
                }
                val => result.push_str(&format!("{}{:?}{}", ctx.indent_str(), val, ctx.nl)),
            };
        }
        Operation::Add(lhs, rhs) => {
            pretty_ssa_2ary((lhs, rhs), ctx, "+", result);
        }
        Operation::Sub(lhs, rhs) => {
            pretty_ssa_2ary((lhs, rhs), ctx, "-", result);
        }
        Operation::Mul(lhs, rhs) => {
            pretty_ssa_2ary((lhs, rhs), ctx, "*", result);
        }
        Operation::Call(func, args) => {
            pretty_ssa_prefix(ctx, result);
            result.push_str(&format!(
                "f{}(",
                ctx.shared.borrow().fns.get(&func.0).unwrap()
            ));
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                match ctx.graph.node(arg).op() {
                    Operation::Value(SpannedMirValue {
                        value: MirValue::Variable(_, pos, _),
                        ..
                    }) => result.push_str(&format!("x{}", pos)),
                    _ => pretty_rec(*arg, &mut ctx.with_indent(0).with_nl(""), result),
                }
            }
            pretty_ssa_suffix(ctx, result);
        }
        op => result.push_str(&format!("{}{:?}\n", ctx.indent_str(), op)),
    }*/
}

fn pretty_ssa_prefix(ctx: &mut PrettyCtx, result: &mut String) {
    result.push_str(&ctx.indent_str());
    ctx.increment_var_count();
    result.push_str(&format!("let x{} = ", ctx.shared.borrow().var_count));
}

fn pretty_ssa_suffix(ctx: &mut PrettyCtx, result: &mut String) {
    result.push_str(&format!(";\n{}", if ctx.in_block { "" } else { ctx.nl }));
}

fn pretty_ssa_2ary(
    (lhs, rhs): (&Link<NodeType>, &Link<NodeType>),
    ctx: &mut PrettyCtx,
    op_str: &str,
    result: &mut String,
) {
    pretty_ssa_prefix(ctx, result);
    pretty_rec(lhs.clone(), &mut ctx.add_indent(1).with_nl(""), result);
    result.push_str(&format!(" {} ", op_str));
    pretty_rec(rhs.clone(), &mut ctx.add_indent(1).with_nl(""), result);
    pretty_ssa_suffix(ctx, result);
}
