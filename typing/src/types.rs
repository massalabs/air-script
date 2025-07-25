use crate::{TypeError, Typing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Felt,
    Bool,
    Int,
}

impl core::fmt::Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Felt => f.write_str("felt"),
            Self::Bool => f.write_str("bool"),
            Self::Int => f.write_str("int"),
        }
    }
}

#[macro_export]
macro_rules! sty {
    (_) => {
        None
    };
    (felt) => {
        Some(ScalarType::Felt)
    };
    (bool) => {
        Some(ScalarType::Bool)
    };
    (int) => {
        Some(ScalarType::Int)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    // annotation: sty
    // where sty is the scalar type
    Scalar(Option<ScalarType>),
    // annotation: `sty[len]`
    // where len is the number of elements in the vector,
    // and sty is the scalar type
    Vector(Option<ScalarType>, usize),
    // annotation: `sty[rows, cols]`
    // where rows and cols are the dimensions of the matrix,
    // and sty is the scalar type
    Matrix(Option<ScalarType>, usize, usize),
}

impl core::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(None) => f.write_str("_"),
            Self::Vector(None, len) => write!(f, "_[{len}]"),
            Self::Matrix(None, rows, cols) => write!(f, "_[{rows}, {cols}]"),
            Self::Scalar(Some(sty)) => f.write_str(&sty.to_string()),
            Self::Vector(Some(sty), len) => write!(f, "{sty}[{len}]"),
            Self::Matrix(Some(sty), rows, cols) => write!(f, "{sty}[{rows}, {cols}]"),
        }
    }
}

#[macro_export]
macro_rules! ty {
    (?) => {
        None::<Type>
    };
    (_) => {
        Some(Type::Scalar(None))
    };
    ($sty:ident) => {
        Some(Type::Scalar(sty!($sty)))
    };
    (_[$len:expr]) => {
        Some(Type::Vector(sty!(_), $len))
    };
    ($sty:ident[$len:expr]) => {
        Some(Type::Vector(sty!($sty), $len))
    };
    (_[$rows:expr, $cols:expr]) => {
        Some(Type::Matrix(sty!(_), $rows, $cols))
    };
    ($sty:ident[$rows:expr, $cols:expr]) => {
        Some(Type::Matrix(sty!($sty), $rows, $cols))
    };
}

pub struct Push(Vec<Option<Type>>);
impl Push {
    pub fn push(mut self, ty: Option<Type>) -> Self {
        self.0.push(ty);
        self
    }
}

#[macro_export]
macro_rules! tys {
    ([$($args:tt)+]) => {
        tys!(RES: Push(vec![]); $($args)+).0
    };
    (RES: $res:expr; ) => {
        $res
    };
    (RES: $res:expr; ?) => {
        tys!(RES: Push::push($res, ty!(?));)
    };
    (RES: $res:expr; _$([$($spec:tt)+])? $(, $($rest:tt)+)?) => {
        tys!(RES: Push::push($res, ty!(_$([$($spec)+])?)); $($($rest)+)?)
    };
    (RES: $res:expr; $name:ident$([$($spec:tt)+])? $(, $($rest:tt)+)?) => {
        tys!(RES: Push::push($res, ty!($name$([$($spec)+])?)); $($($rest)+)?)
    };
}

#[macro_export]
macro_rules! tty {
    ([$($n1:ident$([$l1:literal])?),*]) => {
        Vec::<Option<Type>>::from([
            $(tty!($n1$([$l1])?)),*
        ])
    };
    ($name:ident[$len:literal]) => {
        ty!(felt[$len])
    };
    ($name:ident) => {
        ty!(felt[1])
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionType {
    Evaluator(Vec<Option<Type>>),
    Function(Vec<Option<Type>>, Option<Type>),
}

impl FunctionType {
    pub fn args(&self) -> &[Option<Type>] {
        match self {
            Self::Evaluator(args) => args,
            Self::Function(args, _) => args,
        }
    }

    pub fn ret(&self) -> Option<Type> {
        match self {
            Self::Evaluator(_) => None,
            Self::Function(_, ret) => *ret,
        }
    }
}

impl core::fmt::Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Evaluator(args) => {
                f.write_str("ev(")?;
                write!(
                    f,
                    "[{}]",
                    args.iter().map(|ty| ty.show_ty().to_string()).collect::<Vec<_>>().join(", ")
                )?;
                f.write_str(")")
            },
            Self::Function(args, ret) => {
                f.write_str("fn(")?;
                f.write_str(
                    &args.iter().map(|ty| ty.show_ty().to_string()).collect::<Vec<_>>().join(", "),
                )?;
                f.write_str(") -> ")?;
                if let Some(ret_type) = ret {
                    write!(f, "{}", ret_type)
                } else {
                    f.write_str("?")
                }
            },
        }
    }
}

#[macro_export]
macro_rules! fty {
    (ev ([])) => {
        FunctionType::Evaluator(vec![])
    };
    (ev ([$($tty:tt)+])) => {
        FunctionType::Evaluator(tty!([$($tty)+]))
    };
    (fn ($($arg:tt)*) -> $($ret:tt)+) => {
        FunctionType::Function(tys!([$($arg)*]), ty!($($ret)+))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinType {
    Eq(Option<Type>, Option<Type>, Option<Type>),
    Add(Option<Type>, Option<Type>, Option<Type>),
    Sub(Option<Type>, Option<Type>, Option<Type>),
    Mul(Option<Type>, Option<Type>, Option<Type>),
    Exp(Option<Type>, Option<Type>, Option<Type>),
}

impl BinType {
    pub fn lhs(&self) -> Option<Type> {
        match self {
            Self::Eq(lhs, _, _)
            | Self::Add(lhs, _, _)
            | Self::Sub(lhs, _, _)
            | Self::Mul(lhs, _, _)
            | Self::Exp(lhs, _, _) => *lhs,
        }
    }

    pub fn lhs_mut(&mut self) -> &mut Option<Type> {
        match self {
            Self::Eq(lhs, _, _)
            | Self::Add(lhs, _, _)
            | Self::Sub(lhs, _, _)
            | Self::Mul(lhs, _, _)
            | Self::Exp(lhs, _, _) => lhs,
        }
    }

    pub fn rhs(&self) -> Option<Type> {
        match self {
            Self::Eq(_, rhs, _)
            | Self::Add(_, rhs, _)
            | Self::Sub(_, rhs, _)
            | Self::Mul(_, rhs, _)
            | Self::Exp(_, rhs, _) => *rhs,
        }
    }

    pub fn rhs_mut(&mut self) -> &mut Option<Type> {
        match self {
            Self::Eq(_, rhs, _)
            | Self::Add(_, rhs, _)
            | Self::Sub(_, rhs, _)
            | Self::Mul(_, rhs, _)
            | Self::Exp(_, rhs, _) => rhs,
        }
    }

    pub fn ret(&self) -> Option<Type> {
        match self {
            Self::Eq(_, _, ret)
            | Self::Add(_, _, ret)
            | Self::Sub(_, _, ret)
            | Self::Mul(_, _, ret)
            | Self::Exp(_, _, ret) => *ret,
        }
    }

    pub fn ret_mut(&mut self) -> &mut Option<Type> {
        match self {
            Self::Eq(_, _, ret)
            | Self::Add(_, _, ret)
            | Self::Sub(_, _, ret)
            | Self::Mul(_, _, ret)
            | Self::Exp(_, _, ret) => ret,
        }
    }

    pub fn as_fn(&self) -> FunctionType {
        match self {
            Self::Eq(lhs, rhs, ret)
            | Self::Add(lhs, rhs, ret)
            | Self::Sub(lhs, rhs, ret)
            | Self::Mul(lhs, rhs, ret)
            | Self::Exp(lhs, rhs, ret) => FunctionType::Function(vec![*lhs, *rhs], *ret),
        }
    }
    pub fn without_shape(&self) -> Self {
        match self {
            Self::Eq(lhs, rhs, ret) => Self::Eq(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Add(lhs, rhs, ret) => Self::Add(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Sub(lhs, rhs, ret) => Self::Sub(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Mul(lhs, rhs, ret) => Self::Mul(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
            Self::Exp(lhs, rhs, ret) => Self::Exp(
                (*lhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*rhs).and_then(|ty| ty.ty_with_shape(ty!(_))),
                (*ret).and_then(|ty| ty.ty_with_shape(ty!(_))),
            ),
        }
    }
}

impl core::fmt::Display for BinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq(lhs, rhs, None) => write!(f, "{} = {}", lhs.show_ty(), rhs.show_ty()),
            Self::Add(lhs, rhs, None) => write!(f, "{} + {}", lhs.show_ty(), rhs.show_ty()),
            Self::Sub(lhs, rhs, None) => write!(f, "{} - {}", lhs.show_ty(), rhs.show_ty()),
            Self::Mul(lhs, rhs, None) => write!(f, "{} * {}", lhs.show_ty(), rhs.show_ty()),
            Self::Exp(lhs, rhs, None) => write!(f, "{} ^ {}", lhs.show_ty(), rhs.show_ty()),
            Self::Eq(lhs, rhs, ret) => {
                write!(f, "{} = {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Add(lhs, rhs, ret) => {
                write!(f, "{} + {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Sub(lhs, rhs, ret) => {
                write!(f, "{} - {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Mul(lhs, rhs, ret) => {
                write!(f, "{} * {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
            Self::Exp(lhs, rhs, ret) => {
                write!(f, "{} ^ {} -> {}", lhs.show_ty(), rhs.show_ty(), ret.show_ty())
            },
        }
    }
}

#[macro_export]
macro_rules! bty {
    ($($bty:tt)+ -> $($ret:tt)+) => {{
        let b = bty!($($bty)+);
        b.ret_mut().replace(ty!($($ret)+));
        b
    }};
    (? = $($rhs:tt)+) => {
        BinType::Eq(ty!(?), ty!($($rhs)+), ty!(?))
    };
    (_$([$($spec:tt)+])? = $($rhs:tt)+) => {
        BinType::Eq(ty!(_$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? = $($rhs:tt)+) => {
        BinType::Eq(ty!($sty$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    (? + $($rhs:tt)+) => {
        BinType::Add(ty!(?), ty!($($rhs)+), ty!(?))
    };
    (_$([$($spec:tt)+])? + $($rhs:tt)+) => {
        BinType::Add(ty!(_$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? + $($rhs:tt)+) => {
        BinType::Add(ty!($sty$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    (? - $($rhs:tt)+) => {
        BinType::Sub(ty!(?), ty!($($rhs)+), ty!(?))
    };
    (_$([$($spec:tt)+])? - $($rhs:tt)+) => {
        BinType::Sub(ty!(_$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? - $($rhs:tt)+) => {
        BinType::Sub(ty!($sty$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    (? * $($rhs:tt)+) => {
        BinType::Mul(ty!(?), ty!($($rhs)+), ty!(?))
    };
    (_$([$($spec:tt)+])? * $($rhs:tt)+) => {
        BinType::Mul(ty!(_$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? * $($rhs:tt)+) => {
        BinType::Mul(ty!($sty$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    (? ^ $($rhs:tt)+) => {
        BinType::Exp(ty!(?), ty!($($rhs)+), ty!(?))
    };
    (_$([$($spec:tt)+])? ^ $($rhs:tt)+) => {
        BinType::Exp(ty!(_$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
    ($sty:ident$([$($spec:tt)+])? ^ $($rhs:tt)+) => {
        BinType::Exp(ty!($sty$([$($spec)+])?), ty!($($rhs)+), ty!(?))
    };
}

impl BinType {
    pub fn infer_bin_ty_eq(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.ret() {
            return Ok(Some(ret));
        }
        let lhs = self.lhs();
        let rhs = self.rhs();
        if !lhs.is_shape_compatible(&rhs) {
            return Err(TypeError::IncompatibleShapes { ty: lhs, new_ty: rhs });
        }
        match self.without_shape() {
            bty!(? = ?) => Ok(ty!(bool)),
            bty!(? = _) => Ok(ty!(bool)),
            bty!(? = felt) => Ok(ty!(bool)),
            bty!(? = int) => Ok(ty!(bool)),
            bty!(? = bool) => Ok(ty!(bool)),
            bty!(_ = ?) => Ok(ty!(bool)),
            bty!(_ = _) => Ok(ty!(bool)),
            bty!(_ = felt) => Ok(ty!(bool)),
            bty!(_ = int) => Ok(ty!(bool)),
            bty!(_ = bool) => Ok(ty!(bool)),
            bty!(felt = ?) => Ok(ty!(bool)),
            bty!(felt = _) => Ok(ty!(bool)),
            bty!(felt = felt) => Ok(ty!(bool)),
            bty!(felt = int) => Ok(ty!(bool)),
            bty!(felt = bool) => Ok(ty!(bool)),
            bty!(int = ?) => Ok(ty!(bool)),
            bty!(int = _) => Ok(ty!(bool)),
            bty!(int = felt) => Ok(ty!(bool)),
            bty!(int = int) => Ok(ty!(bool)),
            bty!(int = bool) => Ok(ty!(bool)),
            bty!(bool = ?) => Ok(ty!(bool)),
            bty!(bool = _) => Ok(ty!(bool)),
            bty!(bool = felt) => Ok(ty!(bool)),
            bty!(bool = int) => Ok(ty!(bool)),
            bty!(bool = bool) => Ok(ty!(bool)),
            _ => Err(TypeError::IncompatibleBinOp { bin_ty: *self }),
        }
    }

    pub fn infer_bin_ty_add(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.ret() {
            return Ok(Some(ret));
        }
        if !self.lhs().is_scalar() || !self.rhs().is_scalar() {
            return Err(TypeError::IncompatibleBinOp { bin_ty: *self });
        }
        match self {
            bty!(? + ?) => Ok(ty!(?)),
            bty!(? + _) => Ok(ty!(?)),
            bty!(? + felt) => Ok(ty!(?)),
            bty!(? + int) => Ok(ty!(?)),
            bty!(? + bool) => Ok(ty!(?)),
            bty!(_ + ?) => Ok(ty!(?)),
            bty!(_ + _) => Ok(ty!(?)),
            bty!(_ + felt) => Ok(ty!(?)),
            bty!(_ + int) => Ok(ty!(?)),
            bty!(_ + bool) => Ok(ty!(?)),
            bty!(felt + ?) => Ok(ty!(?)),
            bty!(felt + _) => Ok(ty!(?)),
            bty!(felt + felt) => Ok(ty!(?)),
            bty!(felt + int) => Ok(ty!(?)),
            bty!(felt + bool) => Ok(ty!(?)),
            bty!(int + ?) => Ok(ty!(?)),
            bty!(int + _) => Ok(ty!(?)),
            bty!(int + felt) => Ok(ty!(?)),
            bty!(int + int) => Ok(ty!(?)),
            bty!(int + bool) => Ok(ty!(?)),
            bty!(bool + ?) => Ok(ty!(?)),
            bty!(bool + _) => Ok(ty!(?)),
            bty!(bool + felt) => Ok(ty!(?)),
            bty!(bool + int) => Ok(ty!(?)),
            bty!(bool + bool) => Ok(ty!(?)),
            _ => Err(TypeError::IncompatibleBinOp { bin_ty: *self }),
        }
    }

    pub fn infer_bin_ty_sub(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.ret() {
            return Ok(Some(ret));
        }
        if !self.lhs().is_scalar() || !self.rhs().is_scalar() {
            return Err(TypeError::IncompatibleBinOp { bin_ty: *self });
        }
        match self {
            bty!(? - ?) => Ok(ty!(?)),
            bty!(? - _) => Ok(ty!(?)),
            bty!(? - felt) => Ok(ty!(?)),
            bty!(? - int) => Ok(ty!(?)),
            bty!(? - bool) => Ok(ty!(?)),
            bty!(_ - ?) => Ok(ty!(?)),
            bty!(_ - _) => Ok(ty!(?)),
            bty!(_ - felt) => Ok(ty!(?)),
            bty!(_ - int) => Ok(ty!(?)),
            bty!(_ - bool) => Ok(ty!(?)),
            bty!(felt - ?) => Ok(ty!(?)),
            bty!(felt - _) => Ok(ty!(?)),
            bty!(felt - felt) => Ok(ty!(?)),
            bty!(felt - int) => Ok(ty!(?)),
            bty!(felt - bool) => Ok(ty!(?)),
            bty!(int - ?) => Ok(ty!(?)),
            bty!(int - _) => Ok(ty!(?)),
            bty!(int - felt) => Ok(ty!(?)),
            bty!(int - int) => Ok(ty!(?)),
            bty!(int - bool) => Ok(ty!(?)),
            bty!(bool - ?) => Ok(ty!(?)),
            bty!(bool - _) => Ok(ty!(?)),
            bty!(bool - felt) => Ok(ty!(?)),
            bty!(bool - int) => Ok(ty!(?)),
            bty!(bool - bool) => Ok(ty!(?)),
            _ => Err(TypeError::IncompatibleBinOp { bin_ty: *self }),
        }
    }

    pub fn infer_bin_ty_mul(&self) -> Result<Option<Type>, TypeError> {
        if let Some(ret) = self.ret() {
            return Ok(Some(ret));
        }
        if !self.lhs().is_scalar() || !self.rhs().is_scalar() {
            return Err(TypeError::IncompatibleBinOp { bin_ty: *self });
        }
        match self {
            bty!(? * ?) => Ok(ty!(?)),
            bty!(? * _) => Ok(ty!(?)),
            bty!(? * felt) => Ok(ty!(?)),
            bty!(? * int) => Ok(ty!(?)),
            bty!(? * bool) => Ok(ty!(?)),
            bty!(_ * ?) => Ok(ty!(?)),
            bty!(_ * _) => Ok(ty!(?)),
            bty!(_ * felt) => Ok(ty!(?)),
            bty!(_ * int) => Ok(ty!(?)),
            bty!(_ * bool) => Ok(ty!(?)),
            bty!(felt * ?) => Ok(ty!(?)),
            bty!(felt * _) => Ok(ty!(?)),
            bty!(felt * felt) => Ok(ty!(?)),
            bty!(felt * int) => Ok(ty!(?)),
            bty!(felt * bool) => Ok(ty!(?)),
            bty!(int * ?) => Ok(ty!(?)),
            bty!(int * _) => Ok(ty!(?)),
            bty!(int * felt) => Ok(ty!(?)),
            bty!(int * int) => Ok(ty!(?)),
            bty!(int * bool) => Ok(ty!(?)),
            bty!(bool * ?) => Ok(ty!(?)),
            bty!(bool * _) => Ok(ty!(?)),
            bty!(bool * felt) => Ok(ty!(?)),
            bty!(bool * int) => Ok(ty!(?)),
            bty!(bool * bool) => Ok(ty!(?)),
            _ => Err(TypeError::IncompatibleBinOp { bin_ty: *self }),
        }
    }

    pub fn infer_bin_ty_exp(&self) -> Result<Option<Type>, TypeError> {
        if !self.lhs().is_scalar() || !self.rhs().is_scalar_int() {
            return Err(TypeError::IncompatibleBinOp { bin_ty: *self });
        }
        // a bool to any power is still a bool:
        //   - 0^(n) = 0
        //   - 1^(n*2) = 1
        //   - 1^(n*2+1) = 0
        // a felt to any power is still a felt
        // an int to any power is still an int
        // a ? to any power is still a ?
        Ok(self.lhs())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Value(Option<Type>),
    Callable(FunctionType),
}

impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(ty) => write!(f, "{}", ty.show_ty()),
            Self::Callable(fty) => write!(f, "{}", fty),
        }
    }
}

#[macro_export]
macro_rules! kind {
    (ev $($spec:tt)+) => {
        Kind::Callable(fty!(ev $($spec)+))
    };
    (fn ($($args:tt)*) -> $($ret:tt)+) => {
        Kind::Callable(fty!(fn ($($args)*) -> $($ret)+))
    };
    ($($spec:tt)+) => {
        Kind::Value(ty!($($spec)+))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_scalar_type() {
        assert_eq!(sty!(_), None::<ScalarType>);
        assert_eq!(sty!(felt), Some(ScalarType::Felt));
        assert_eq!(sty!(bool), Some(ScalarType::Bool));
        assert_eq!(sty!(int), Some(ScalarType::Int));
    }

    #[test]
    fn test_macro_type() {
        assert_eq!(ty!(?), None::<Type>);
        assert_eq!(ty!(_), Some(Type::Scalar(None)));
        assert_eq!(ty!(felt), Some(Type::Scalar(Some(ScalarType::Felt))));
        assert_eq!(ty!(bool), Some(Type::Scalar(Some(ScalarType::Bool))));
        assert_eq!(ty!(int), Some(Type::Scalar(Some(ScalarType::Int))));
        assert_eq!(ty!(_[5]), Some(Type::Vector(None, 5)));
        assert_eq!(ty!(int[5]), Some(Type::Vector(Some(ScalarType::Int), 5)));
        assert_eq!(ty!(_[3, 4]), Some(Type::Matrix(None, 3, 4)));
        assert_eq!(ty!(felt[3, 4]), Some(Type::Matrix(Some(ScalarType::Felt), 3, 4)));
    }

    #[test]
    fn test_macro_trace_segment_type() {
        assert_eq!(tty!(a), ty!(felt[1]));
        assert_eq!(tty!(a[5]), ty!(felt[5]));
        assert_eq!(tty!([]), Vec::<Option<Type>>::new());
        assert_eq!(tty!([a]), vec![ty!(felt[1])]);
        assert_eq!(tty!([a[5]]), vec![ty!(felt[5])]);
        assert_eq!(tty!([a[1], b[3]]), vec![ty!(felt[1]), ty!(felt[3])]);
    }

    #[test]
    fn test_macro_function_type() {
        assert_eq!(fty!(ev([])), FunctionType::Evaluator(vec![]));
        assert_eq!(fty!(ev([a])), FunctionType::Evaluator(vec![ty!(felt[1])]));
        assert_eq!(fty!(ev([a[5]])), FunctionType::Evaluator(vec![ty!(felt[5])]));
        assert_eq!(fty!(ev([a, b[3]])), FunctionType::Evaluator(vec![ty!(felt[1]), ty!(felt[3])]));
        assert_eq!(
            fty!(ev([a[1], b[3]])),
            FunctionType::Evaluator(vec![ty!(felt[1]), ty!(felt[3])])
        );
        assert_eq!(
            fty!(ev([a[1], b[3]])),
            FunctionType::Evaluator(vec![ty!(felt[1]), ty!(felt[3])])
        );

        assert_eq!(fty!(fn(int) -> felt), FunctionType::Function(vec![ty!(int)], ty!(felt)));
        assert_eq!(
            fty!(fn(int[5]) -> felt[3, 4]),
            FunctionType::Function(vec![ty!(int[5])], ty!(felt[3, 4]),)
        );
        assert_eq!(
            fty!(fn(int[5], felt) -> felt[3, 4]),
            FunctionType::Function(vec![ty!(int[5]), ty!(felt)], ty!(felt[3, 4]),)
        );
        assert_eq!(
            fty!(fn(int[5], felt, bool[3, 4]) -> felt[3, 4]),
            FunctionType::Function(vec![ty!(int[5]), ty!(felt), ty!(bool[3, 4]),], ty!(felt[3, 4]),)
        );
    }

    #[test]
    fn test_macro_bin_type() {
        assert_eq!(bty!(int + felt), BinType::Add(ty!(int), ty!(felt), ty!(?)));
        assert_eq!(bty!(_ - felt), BinType::Sub(ty!(_), ty!(felt), ty!(?)));
        assert_eq!(bty!(? = felt), BinType::Eq(ty!(?), ty!(felt), ty!(?)));
        assert_eq!(bty!(int + ?), BinType::Add(ty!(int), ty!(?), ty!(?)));
        assert_eq!(bty!(int - felt), BinType::Sub(ty!(int), ty!(felt), ty!(?)));
        assert_eq!(bty!(int[2] * felt[2]), BinType::Mul(ty!(int[2]), ty!(felt[2]), ty!(?)));
        assert_eq!(bty!(int[2, 3] ^ _), BinType::Exp(ty!(int[2, 3]), ty!(_), ty!(?)));
        assert_eq!(bty!(bool[5] = _[5]), BinType::Eq(ty!(bool[5]), ty!(_[5]), ty!(?)));
    }

    #[test]
    fn test_macro_kind() {
        assert_eq!(kind!(ev([])), Kind::Callable(fty!(ev([]))));
        assert_eq!(kind!(ev([a])), Kind::Callable(fty!(ev([a]))));
        assert_eq!(kind!(fn(int) -> felt), Kind::Callable(fty!(fn(int) -> felt)));
        assert_eq!(kind!(int), Kind::Value(ty!(int)));
        assert_eq!(kind!(_), Kind::Value(ty!(_)));
        assert_eq!(kind!(bool[3, 4]), Kind::Value(ty!(bool[3, 4])));
    }
}
