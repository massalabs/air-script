mod types;
pub use types::*;

pub trait Typing {
    fn ty(&self) -> Option<Type>;
    fn scalar_ty(&self) -> Option<ScalarType> {
        self.ty().scalar_ty()
    }
    fn is_scalar_int(&self) -> bool {
        matches!(self.scalar_ty(), sty!(int))
    }
    fn is_scalar_felt(&self) -> bool {
        matches!(self.scalar_ty(), sty!(felt))
    }
    fn is_scalar_bool(&self) -> bool {
        matches!(self.scalar_ty(), sty!(bool))
    }
    fn is_scalar(&self) -> bool {
        matches!(self.ty(), Some(Type::Scalar(_)))
    }
    fn is_vector(&self) -> bool {
        matches!(self.ty(), Some(Type::Vector(_, _)))
    }
    fn is_matrix(&self) -> bool {
        matches!(self.ty(), Some(Type::Matrix(_, _, _)))
    }
    /// Returns true if `self` is a subtype of `other`
    /// _ <= bool < felt < int <= _
    /// self\\other | _ | int | felt | bool |
    /// ------------|---|-----|------|------|
    /// _           | y |   y |    y |    y |
    /// int         | y |   y |    n |    n |
    /// felt        | y |   y |    y |    n |
    /// bool        | y |   y |    y |    y |
    fn is_scalar_subtype(&self, other: &impl Typing) -> bool {
        !matches!(
            (self.scalar_ty(), other.scalar_ty()),
            (sty!(int), sty!(felt) | sty!(bool)) | (sty!(felt), sty!(bool))
        )
    }
    /// Returns true if `self` and `other` are compatible scalar types.
    /// This means `self` is a subtype of `other` or they are convertible.
    /// _ <= felt = bool < int <= _
    /// self\\other | _ | int | bool | felt |
    /// ------------|---|-----|------|------|
    /// _           | y |   y |    y |    y |
    /// int         | y |   y |    n |    n |
    /// bool        | y |   y |    y |    y |
    /// felt        | y |   y |    y |    y |
    /// NOTE: Conversion from felt to bool is allowed,
    /// but should raise a diagnostic if not associated with
    /// a `enf x^2 = x` transition constraint.
    fn is_scalar_compatible(&self, other: &impl Typing) -> bool {
        self.is_scalar_subtype(other)
            || matches!((self.scalar_ty(), other.scalar_ty()), (sty!(felt), sty!(bool)))
    }
    fn is_shape_compatible(&self, other: &impl Typing) -> bool {
        match (self.ty(), other.ty()) {
            (None, _) | (_, None) => true,
            (Some(Type::Scalar(_)), Some(Type::Scalar(_))) => true,
            (Some(Type::Vector(_, len1)), Some(Type::Vector(_, len2))) => {
                len1 == len2 || len1 == u32::MAX as usize || len2 == u32::MAX as usize
            },
            (Some(Type::Matrix(_, rows1, cols1)), Some(Type::Matrix(_, rows2, cols2))) => {
                (rows1 == rows2 || rows1 == u32::MAX as usize || rows2 == u32::MAX as usize)
                    && (cols1 == cols2 || cols1 == u32::MAX as usize || cols2 == u32::MAX as usize)
            },
            _ => false,
        }
    }
    /// WARNING: This check assumes covariance for container types.
    /// If A is a subtype of B, then A[..] is a subtype of B[..]
    /// This is only true for *immutable* containers!
    /// If we ever support mutable containers, this will need to be revisited.
    /// Conversion between from felt to bool is allowed,
    /// but should raise a diagnostic if not associated with
    /// a `enf x^2 = x` transition constraint.
    ///
    /// For example:
    /// ```ignore
    /// let a: int[5] = [1, 2, 3, 4, 5];
    /// let b: felt[5] = a; // This is valid because felt is a subtype of int
    /// let c: bool[5] = a; // This is valid because bool is a subtype of int
    /// let d: int[5] = b; // Error: int is not a subtype of felt
    /// let e: bool[5] = b; // Error: missing constraint `enf [x^2 = x for x in e]`
    /// let f: felt[5] = c; // This is valid because felt is a subtype of bool
    /// ```
    /// Returns true if `self` is a subtype of `other`
    fn is_compatible(&self, other: &impl Typing) -> bool {
        if !self.is_shape_compatible(other) {
            return false;
        }
        self.is_scalar_compatible(other)
    }
    /// WARNING: This check assumes covariance for container types.
    /// If A is a subtype of B, then A[..] is a subtype of B[..]
    /// This is only true for *immutable* containers!
    /// If we ever support mutable containers, this will need to be revisited.
    /// See the documentation for `is_compatible` for examples.
    ///
    /// Returns true if `self` is a subtype of `other`
    fn is_subtype(&self, other: &impl Typing) -> bool {
        if !self.is_shape_compatible(other) {
            return false;
        }
        self.is_scalar_subtype(other)
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

impl Typing for BinType {
    fn ty(&self) -> Option<Type> {
        todo!()
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

    #[test]
    fn test_typing_subtype() {
        assert!(ty!().is_subtype(&ty!()));
        assert!(ty!().is_subtype(&ty!(_)));
        assert!(ty!().is_subtype(&ty!(int)));
        assert!(ty!().is_subtype(&ty!(felt)));
        assert!(ty!().is_subtype(&ty!(bool)));
        assert!(ty!().is_subtype(&ty!(_[5])));
        assert!(ty!().is_subtype(&ty!(int[5])));
        assert!(ty!().is_subtype(&ty!(felt[5])));
        assert!(ty!().is_subtype(&ty!(bool[5])));
        assert!(ty!().is_subtype(&ty!(_[3, 4])));
        assert!(ty!().is_subtype(&ty!(int[3, 4])));
        assert!(ty!().is_subtype(&ty!(felt[3, 4])));
        assert!(ty!().is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(_).is_subtype(&ty!()));
        assert!(ty!(_).is_subtype(&ty!(_)));
        assert!(ty!(_).is_subtype(&ty!(int)));
        assert!(ty!(_).is_subtype(&ty!(felt)));
        assert!(ty!(_).is_subtype(&ty!(bool)));
        assert!(!ty!(_).is_subtype(&ty!(_[5])));
        assert!(!ty!(_).is_subtype(&ty!(int[5])));
        assert!(!ty!(_).is_subtype(&ty!(felt[5])));
        assert!(!ty!(_).is_subtype(&ty!(bool[5])));
        assert!(!ty!(_).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(_).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(_).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(_).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(int).is_subtype(&ty!()));
        assert!(ty!(int).is_subtype(&ty!(_)));
        assert!(ty!(int).is_subtype(&ty!(int)));
        assert!(!ty!(int).is_subtype(&ty!(felt)));
        assert!(!ty!(int).is_subtype(&ty!(bool)));
        assert!(!ty!(int).is_subtype(&ty!(_[5])));
        assert!(!ty!(int).is_subtype(&ty!(int[5])));
        assert!(!ty!(int).is_subtype(&ty!(felt[5])));
        assert!(!ty!(int).is_subtype(&ty!(bool[5])));
        assert!(!ty!(int).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(int).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(int).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(int).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(felt).is_subtype(&ty!()));
        assert!(ty!(felt).is_subtype(&ty!(_)));
        assert!(ty!(felt).is_subtype(&ty!(int)));
        assert!(ty!(felt).is_subtype(&ty!(felt)));
        assert!(!ty!(felt).is_subtype(&ty!(bool)));
        assert!(!ty!(felt).is_subtype(&ty!(_[5])));
        assert!(!ty!(felt).is_subtype(&ty!(int[5])));
        assert!(!ty!(felt).is_subtype(&ty!(felt[5])));
        assert!(!ty!(felt).is_subtype(&ty!(bool[5])));
        assert!(!ty!(felt).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(felt).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(felt).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(felt).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(bool).is_subtype(&ty!()));
        assert!(ty!(bool).is_subtype(&ty!(_)));
        assert!(ty!(bool).is_subtype(&ty!(int)));
        assert!(ty!(bool).is_subtype(&ty!(felt)));
        assert!(ty!(bool).is_subtype(&ty!(bool)));
        assert!(!ty!(bool).is_subtype(&ty!(_[5])));
        assert!(!ty!(bool).is_subtype(&ty!(int[5])));
        assert!(!ty!(bool).is_subtype(&ty!(felt[5])));
        assert!(!ty!(bool).is_subtype(&ty!(bool[5])));
        assert!(!ty!(bool).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(bool).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(bool).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(bool).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(_[5]).is_subtype(&ty!()));
        assert!(!ty!(_[5]).is_subtype(&ty!(_)));
        assert!(!ty!(_[5]).is_subtype(&ty!(int)));
        assert!(!ty!(_[5]).is_subtype(&ty!(felt)));
        assert!(!ty!(_[5]).is_subtype(&ty!(bool)));
        assert!(ty!(_[5]).is_subtype(&ty!(_[5])));
        assert!(ty!(_[5]).is_subtype(&ty!(int[5])));
        assert!(ty!(_[5]).is_subtype(&ty!(felt[5])));
        assert!(ty!(_[5]).is_subtype(&ty!(bool[5])));
        assert!(!ty!(_[5]).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(_[5]).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(_[5]).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(_[5]).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(int[5]).is_subtype(&ty!()));
        assert!(!ty!(int[5]).is_subtype(&ty!(_)));
        assert!(!ty!(int[5]).is_subtype(&ty!(int)));
        assert!(!ty!(int[5]).is_subtype(&ty!(felt)));
        assert!(!ty!(int[5]).is_subtype(&ty!(bool)));
        assert!(ty!(int[5]).is_subtype(&ty!(_[5])));
        assert!(ty!(int[5]).is_subtype(&ty!(int[5])));
        assert!(!ty!(int[5]).is_subtype(&ty!(felt[5])));
        assert!(!ty!(int[5]).is_subtype(&ty!(bool[5])));
        assert!(!ty!(int[5]).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(int[5]).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(int[5]).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(int[5]).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(felt[5]).is_subtype(&ty!()));
        assert!(!ty!(felt[5]).is_subtype(&ty!(_)));
        assert!(!ty!(felt[5]).is_subtype(&ty!(int)));
        assert!(!ty!(felt[5]).is_subtype(&ty!(felt)));
        assert!(!ty!(felt[5]).is_subtype(&ty!(bool)));
        assert!(ty!(felt[5]).is_subtype(&ty!(_[5])));
        assert!(ty!(felt[5]).is_subtype(&ty!(int[5])));
        assert!(ty!(felt[5]).is_subtype(&ty!(felt[5])));
        assert!(!ty!(felt[5]).is_subtype(&ty!(bool[5])));
        assert!(!ty!(felt[5]).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(felt[5]).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(felt[5]).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(felt[5]).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(bool[5]).is_subtype(&ty!()));
        assert!(!ty!(bool[5]).is_subtype(&ty!(_)));
        assert!(!ty!(bool[5]).is_subtype(&ty!(int)));
        assert!(!ty!(bool[5]).is_subtype(&ty!(felt)));
        assert!(!ty!(bool[5]).is_subtype(&ty!(bool)));
        assert!(ty!(bool[5]).is_subtype(&ty!(_[5])));
        assert!(ty!(bool[5]).is_subtype(&ty!(int[5])));
        assert!(ty!(bool[5]).is_subtype(&ty!(felt[5])));
        assert!(ty!(bool[5]).is_subtype(&ty!(bool[5])));
        assert!(!ty!(bool[5]).is_subtype(&ty!(_[3, 4])));
        assert!(!ty!(bool[5]).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(bool[5]).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(bool[5]).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(_[3, 4]).is_subtype(&ty!()));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(_)));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(int)));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(felt)));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(bool)));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(_[5])));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(int[5])));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(felt[5])));
        assert!(!ty!(_[3, 4]).is_subtype(&ty!(bool[5])));
        assert!(ty!(_[3, 4]).is_subtype(&ty!(_[3, 4])));
        assert!(ty!(_[3, 4]).is_subtype(&ty!(int[3, 4])));
        assert!(ty!(_[3, 4]).is_subtype(&ty!(felt[3, 4])));
        assert!(ty!(_[3, 4]).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(int[3, 4]).is_subtype(&ty!()));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(_)));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(int)));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(felt)));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(bool)));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(_[5])));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(int[5])));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(felt[5])));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(bool[5])));
        assert!(ty!(int[3, 4]).is_subtype(&ty!(_[3, 4])));
        assert!(ty!(int[3, 4]).is_subtype(&ty!(int[3, 4])));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(int[3, 4]).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(felt[3, 4]).is_subtype(&ty!()));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(_)));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(int)));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(felt)));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(bool)));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(_[5])));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(int[5])));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(felt[5])));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(bool[5])));
        assert!(ty!(felt[3, 4]).is_subtype(&ty!(_[3, 4])));
        assert!(ty!(felt[3, 4]).is_subtype(&ty!(int[3, 4])));
        assert!(ty!(felt[3, 4]).is_subtype(&ty!(felt[3, 4])));
        assert!(!ty!(felt[3, 4]).is_subtype(&ty!(bool[3, 4])));

        assert!(ty!(bool[3, 4]).is_subtype(&ty!()));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(_)));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(int)));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(felt)));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(bool)));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(_[5])));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(int[5])));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(felt[5])));
        assert!(!ty!(bool[3, 4]).is_subtype(&ty!(bool[5])));
        assert!(ty!(bool[3, 4]).is_subtype(&ty!(_[3, 4])));
        assert!(ty!(bool[3, 4]).is_subtype(&ty!(int[3, 4])));
        assert!(ty!(bool[3, 4]).is_subtype(&ty!(felt[3, 4])));
        assert!(ty!(bool[3, 4]).is_subtype(&ty!(bool[3, 4])));
    }

    #[test]
    fn test_typing_compatible() {
        assert!(ty!().is_compatible(&ty!()));
        assert!(ty!().is_compatible(&ty!(_)));
        assert!(ty!().is_compatible(&ty!(int)));
        assert!(ty!().is_compatible(&ty!(felt)));
        assert!(ty!().is_compatible(&ty!(bool)));
        assert!(ty!().is_compatible(&ty!(_[5])));
        assert!(ty!().is_compatible(&ty!(int[5])));
        assert!(ty!().is_compatible(&ty!(felt[5])));
        assert!(ty!().is_compatible(&ty!(bool[5])));
        assert!(ty!().is_compatible(&ty!(_[3, 4])));
        assert!(ty!().is_compatible(&ty!(int[3, 4])));
        assert!(ty!().is_compatible(&ty!(felt[3, 4])));
        assert!(ty!().is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(_).is_compatible(&ty!()));
        assert!(ty!(_).is_compatible(&ty!(_)));
        assert!(ty!(_).is_compatible(&ty!(int)));
        assert!(ty!(_).is_compatible(&ty!(felt)));
        assert!(ty!(_).is_compatible(&ty!(bool)));
        assert!(!ty!(_).is_compatible(&ty!(_[5])));
        assert!(!ty!(_).is_compatible(&ty!(int[5])));
        assert!(!ty!(_).is_compatible(&ty!(felt[5])));
        assert!(!ty!(_).is_compatible(&ty!(bool[5])));
        assert!(!ty!(_).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(_).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(_).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(_).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(int).is_compatible(&ty!()));
        assert!(ty!(int).is_compatible(&ty!(_)));
        assert!(ty!(int).is_compatible(&ty!(int)));
        assert!(!ty!(int).is_compatible(&ty!(felt)));
        assert!(!ty!(int).is_compatible(&ty!(bool)));
        assert!(!ty!(int).is_compatible(&ty!(_[5])));
        assert!(!ty!(int).is_compatible(&ty!(int[5])));
        assert!(!ty!(int).is_compatible(&ty!(felt[5])));
        assert!(!ty!(int).is_compatible(&ty!(bool[5])));
        assert!(!ty!(int).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(int).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(int).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(int).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(felt).is_compatible(&ty!()));
        assert!(ty!(felt).is_compatible(&ty!(_)));
        assert!(ty!(felt).is_compatible(&ty!(int)));
        assert!(ty!(felt).is_compatible(&ty!(felt)));
        assert!(ty!(felt).is_compatible(&ty!(bool)));
        assert!(!ty!(felt).is_compatible(&ty!(_[5])));
        assert!(!ty!(felt).is_compatible(&ty!(int[5])));
        assert!(!ty!(felt).is_compatible(&ty!(felt[5])));
        assert!(!ty!(felt).is_compatible(&ty!(bool[5])));
        assert!(!ty!(felt).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(felt).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(felt).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(felt).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(bool).is_compatible(&ty!()));
        assert!(ty!(bool).is_compatible(&ty!(_)));
        assert!(ty!(bool).is_compatible(&ty!(int)));
        assert!(ty!(bool).is_compatible(&ty!(felt)));
        assert!(ty!(bool).is_compatible(&ty!(bool)));
        assert!(!ty!(bool).is_compatible(&ty!(_[5])));
        assert!(!ty!(bool).is_compatible(&ty!(int[5])));
        assert!(!ty!(bool).is_compatible(&ty!(felt[5])));
        assert!(!ty!(bool).is_compatible(&ty!(bool[5])));
        assert!(!ty!(bool).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(bool).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(bool).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(bool).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(_[5]).is_compatible(&ty!()));
        assert!(!ty!(_[5]).is_compatible(&ty!(_)));
        assert!(!ty!(_[5]).is_compatible(&ty!(int)));
        assert!(!ty!(_[5]).is_compatible(&ty!(felt)));
        assert!(!ty!(_[5]).is_compatible(&ty!(bool)));
        assert!(ty!(_[5]).is_compatible(&ty!(_[5])));
        assert!(ty!(_[5]).is_compatible(&ty!(int[5])));
        assert!(ty!(_[5]).is_compatible(&ty!(felt[5])));
        assert!(ty!(_[5]).is_compatible(&ty!(bool[5])));
        assert!(!ty!(_[5]).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(_[5]).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(_[5]).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(_[5]).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(int[5]).is_compatible(&ty!()));
        assert!(!ty!(int[5]).is_compatible(&ty!(_)));
        assert!(!ty!(int[5]).is_compatible(&ty!(int)));
        assert!(!ty!(int[5]).is_compatible(&ty!(felt)));
        assert!(!ty!(int[5]).is_compatible(&ty!(bool)));
        assert!(ty!(int[5]).is_compatible(&ty!(_[5])));
        assert!(ty!(int[5]).is_compatible(&ty!(int[5])));
        assert!(!ty!(int[5]).is_compatible(&ty!(felt[5])));
        assert!(!ty!(int[5]).is_compatible(&ty!(bool[5])));
        assert!(!ty!(int[5]).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(int[5]).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(int[5]).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(int[5]).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(felt[5]).is_compatible(&ty!()));
        assert!(!ty!(felt[5]).is_compatible(&ty!(_)));
        assert!(!ty!(felt[5]).is_compatible(&ty!(int)));
        assert!(!ty!(felt[5]).is_compatible(&ty!(felt)));
        assert!(!ty!(felt[5]).is_compatible(&ty!(bool)));
        assert!(ty!(felt[5]).is_compatible(&ty!(_[5])));
        assert!(ty!(felt[5]).is_compatible(&ty!(int[5])));
        assert!(ty!(felt[5]).is_compatible(&ty!(felt[5])));
        assert!(ty!(felt[5]).is_compatible(&ty!(bool[5])));
        assert!(!ty!(felt[5]).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(felt[5]).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(felt[5]).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(felt[5]).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(bool[5]).is_compatible(&ty!()));
        assert!(!ty!(bool[5]).is_compatible(&ty!(_)));
        assert!(!ty!(bool[5]).is_compatible(&ty!(int)));
        assert!(!ty!(bool[5]).is_compatible(&ty!(felt)));
        assert!(!ty!(bool[5]).is_compatible(&ty!(bool)));
        assert!(ty!(bool[5]).is_compatible(&ty!(_[5])));
        assert!(ty!(bool[5]).is_compatible(&ty!(int[5])));
        assert!(ty!(bool[5]).is_compatible(&ty!(felt[5])));
        assert!(ty!(bool[5]).is_compatible(&ty!(bool[5])));
        assert!(!ty!(bool[5]).is_compatible(&ty!(_[3, 4])));
        assert!(!ty!(bool[5]).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(bool[5]).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(bool[5]).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(_[3, 4]).is_compatible(&ty!()));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(_)));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(int)));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(felt)));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(bool)));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(_[5])));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(int[5])));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(felt[5])));
        assert!(!ty!(_[3, 4]).is_compatible(&ty!(bool[5])));
        assert!(ty!(_[3, 4]).is_compatible(&ty!(_[3, 4])));
        assert!(ty!(_[3, 4]).is_compatible(&ty!(int[3, 4])));
        assert!(ty!(_[3, 4]).is_compatible(&ty!(felt[3, 4])));
        assert!(ty!(_[3, 4]).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(int[3, 4]).is_compatible(&ty!()));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(_)));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(int)));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(felt)));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(bool)));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(_[5])));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(int[5])));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(felt[5])));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(bool[5])));
        assert!(ty!(int[3, 4]).is_compatible(&ty!(_[3, 4])));
        assert!(ty!(int[3, 4]).is_compatible(&ty!(int[3, 4])));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(felt[3, 4])));
        assert!(!ty!(int[3, 4]).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(felt[3, 4]).is_compatible(&ty!()));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(_)));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(int)));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(felt)));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(bool)));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(_[5])));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(int[5])));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(felt[5])));
        assert!(!ty!(felt[3, 4]).is_compatible(&ty!(bool[5])));
        assert!(ty!(felt[3, 4]).is_compatible(&ty!(_[3, 4])));
        assert!(ty!(felt[3, 4]).is_compatible(&ty!(int[3, 4])));
        assert!(ty!(felt[3, 4]).is_compatible(&ty!(felt[3, 4])));
        assert!(ty!(felt[3, 4]).is_compatible(&ty!(bool[3, 4])));

        assert!(ty!(bool[3, 4]).is_compatible(&ty!()));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(_)));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(int)));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(felt)));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(bool)));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(_[5])));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(int[5])));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(felt[5])));
        assert!(!ty!(bool[3, 4]).is_compatible(&ty!(bool[5])));
        assert!(ty!(bool[3, 4]).is_compatible(&ty!(_[3, 4])));
        assert!(ty!(bool[3, 4]).is_compatible(&ty!(int[3, 4])));
        assert!(ty!(bool[3, 4]).is_compatible(&ty!(felt[3, 4])));
        assert!(ty!(bool[3, 4]).is_compatible(&ty!(bool[3, 4])));
    }
}
