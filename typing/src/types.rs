#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Int,
    Felt,
    Bool,
}

impl core::fmt::Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => f.write_str("int"),
            Self::Felt => f.write_str("felt"),
            Self::Bool => f.write_str("bool"),
        }
    }
}

#[macro_export]
macro_rules! sty {
    ($(_)?) => {
        None
    };
    (int) => {
        Some(ScalarType::Int)
    };
    (felt) => {
        Some(ScalarType::Felt)
    };
    (bool) => {
        Some(ScalarType::Bool)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Scalar(Option<ScalarType>),
    // annotation: `sty[len]`
    // where len is the number of elements in the vector
    Vector(Option<ScalarType>, usize),
    // annotation: `sty[rows, cols]`
    // where rows and cols are the dimensions of the matrix
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
    () => {
        None::<Type>
    };
    (_) => {
        Some(Type::Scalar(None))
    };
    ($($sty:ident)?) => {
        Some(Type::Scalar(sty!($($sty)?)))
    };
    (_[$len:expr]) => {
        Some(Type::Vector(sty!(_), $len))
    };
    ($($sty:ident)?[$len:expr]) => {
        Some(Type::Vector(sty!($($sty)?), $len))
    };
    (_[$rows:expr, $cols:expr]) => {
        Some(Type::Matrix(sty!(_), $rows, $cols))
    };
    ($($sty:ident)?[$rows:expr, $cols:expr]) => {
        Some(Type::Matrix(sty!($($sty)?), $rows, $cols))
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

#[macro_export]
macro_rules! fty {
    (ev ([])) => {
        FunctionType::Evaluator(vec![])
    };
    (ev ([$($tty:tt)+])) => {
        FunctionType::Evaluator(tty!([$($tty)+]))
    };
    (fn ($($arg:tt),*) -> $ret:tt) => {
        FunctionType::Function(vec![ty!($($arg),*)], ty!($ret))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_scalar_type() {
        assert_eq!(sty!(), None::<ScalarType>);
        assert_eq!(sty!(_), None::<ScalarType>);
        assert_eq!(sty!(int), Some(ScalarType::Int));
        assert_eq!(sty!(felt), Some(ScalarType::Felt));
        assert_eq!(sty!(bool), Some(ScalarType::Bool));
    }

    #[test]
    fn test_macro_type() {
        assert_eq!(ty!(), None::<Type>);
        assert_eq!(ty!(_), Some(Type::Scalar(None)));
        assert_eq!(ty!(int), Some(Type::Scalar(Some(ScalarType::Int))));
        assert_eq!(ty!(felt), Some(Type::Scalar(Some(ScalarType::Felt))));
        assert_eq!(ty!(bool), Some(Type::Scalar(Some(ScalarType::Bool))));
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

        assert_eq!(
            fty!(fn(int) -> felt),
            FunctionType::Function(
                vec![Some(Type::Scalar(Some(ScalarType::Int)))],
                Some(Type::Scalar(Some(ScalarType::Felt)))
            )
        );
    }
}
