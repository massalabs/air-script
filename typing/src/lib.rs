mod types;
pub use types::*;

pub trait Typing {
    fn ty(&self) -> Option<Type>;
    fn scalar_ty(&self) -> Option<ScalarType> {
        self.ty().scalar_ty()
    }
}

impl Typing for Type {
    fn ty(&self) -> Option<Type> {
        Some(*self)
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        match self {
            Type::Scalar(st) => *st,
            Type::Vector(st, _) => *st,
            Type::Matrix(st, _, _) => *st,
        }
    }
}

impl Typing for ScalarType {
    fn ty(&self) -> Option<Type> {
        Some(Type::Scalar(Some(*self)))
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        Some(*self)
    }
}

impl<T> Typing for Option<T>
where
    T: Typing,
{
    fn ty(&self) -> Option<Type> {
        self.as_ref().and_then(|t| t.ty())
    }
    fn scalar_ty(&self) -> Option<ScalarType> {
        self.as_ref().and_then(|t| t.scalar_ty())
    }
}

pub struct Typed<T> {
    pub value: T,
    pub ty: Type,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sty, ty};

    #[test]
    fn test_typing() {
        assert_eq!(ty!().ty(), None);
        assert_eq!(ty!().scalar_ty(), sty!());
        assert_eq!(ty!(_).ty(), Some(Type::Scalar(sty!(_))));
        assert_eq!(ty!(_).scalar_ty(), sty!(_));
        assert_eq!(ty!(int).ty(), Some(Type::Scalar(sty!(int))));
        assert_eq!(ty!(int).scalar_ty(), sty!(int));
        assert_eq!(ty!(felt).ty(), Some(Type::Scalar(sty!(felt))));
        assert_eq!(ty!(felt).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool).ty(), Some(Type::Scalar(sty!(bool))));
        assert_eq!(ty!(bool).scalar_ty(), sty!(bool));
        assert_eq!(ty!(_[5]).ty(), Some(Type::Vector(sty!(_), 5)));
        assert_eq!(ty!(_[5]).scalar_ty(), sty!(_));
        assert_eq!(ty!(int[5]).ty(), Some(Type::Vector(sty!(int), 5)));
        assert_eq!(ty!(int[5]).scalar_ty(), sty!(int));
        assert_eq!(ty!(felt[5]).ty(), Some(Type::Vector(sty!(felt), 5)));
        assert_eq!(ty!(felt[5]).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool[5]).ty(), Some(Type::Vector(sty!(bool), 5)));
        assert_eq!(ty!(bool[5]).scalar_ty(), sty!(bool));
        assert_eq!(ty!(_[3, 4]).ty(), Some(Type::Matrix(sty!(_), 3, 4)));
        assert_eq!(ty!(_[3, 4]).scalar_ty(), sty!(_));
        assert_eq!(ty!(int[3, 4]).ty(), Some(Type::Matrix(sty!(int), 3, 4)));
        assert_eq!(ty!(int[3, 4]).scalar_ty(), sty!(int));
        assert_eq!(ty!(felt[3, 4]).ty(), Some(Type::Matrix(sty!(felt), 3, 4)));
        assert_eq!(ty!(felt[3, 4]).scalar_ty(), sty!(felt));
        assert_eq!(ty!(bool[3, 4]).ty(), Some(Type::Matrix(sty!(bool), 3, 4)));
        assert_eq!(ty!(bool[3, 4]).scalar_ty(), sty!(bool));
    }
}
