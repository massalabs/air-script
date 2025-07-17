use super::*;

pub trait Typed {
    fn ty(&self) -> Option<Type>;
    fn scalar_ty(&self) -> ScalarType {
        self.ty().map_or(ScalarType::Felt, |ty| match ty {
            Type::Scalar(sty) => sty,
            Type::Vector(sty, _) => sty,
            Type::Matrix(sty, _, _) => sty,
        })
    }
}
impl<T: Typed> Typed for Option<T> {
    fn ty(&self) -> Option<Type> {
        self.as_ref().and_then(|t| t.ty())
    }
}
impl<T: Typed, E> Typed for Result<T, E> {
    fn ty(&self) -> Option<Type> {
        self.as_ref().ok().and_then(|t| t.ty())
    }
}

impl Typed for ScalarType {
    fn ty(&self) -> Option<Type> {
        Some(Type::Scalar(*self))
    }
}

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ScalarType {
    /// A field element
    #[default]
    Felt,
    /// A boolean value
    Bool,
    /// A 32-bit integer
    /// Currently used exclusively to access indexed types via indices
    /// These need to be constant under all circumstances!
    Uint,
}

impl core::fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Felt => f.write_str("felt"),
            Self::Bool => f.write_str("bool"),
            Self::Uint => f.write_str("uint"),
        }
    }
}

/// The types of values which can be represented in an AirScript program
#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Type {
    /// A field element
    Scalar(ScalarType),
    /// A vector of N integers
    Vector(ScalarType, usize),
    /// A matrix of N rows and M columns
    Matrix(ScalarType, usize, usize),
}
impl Type {
    /// Returns true if this type is an aggregate
    #[inline]
    pub fn is_aggregate(&self) -> bool {
        match self {
            Self::Scalar(_) => false,
            Self::Vector(..) | Self::Matrix(..) => true,
        }
    }

    /// Returns true if this type is a scalar
    #[inline]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Scalar(_))
    }

    /// Returns true if this type is a valid iterable in a comprehension
    #[inline]
    pub fn is_iterable(&self) -> bool {
        self.is_vector()
    }

    /// Returns true if this type is a vector
    #[inline]
    pub fn is_vector(&self) -> bool {
        matches!(self, Self::Vector(..))
    }

    /// Return a new [Type] representing the type of the value produced by the given [AccessType]
    pub fn access(&self, access_type: AccessType) -> Result<Self, InvalidAccessError> {
        match *self {
            ty if access_type == AccessType::Default => Ok(ty),
            Self::Scalar(_) => Err(InvalidAccessError::IndexIntoScalar),
            Self::Vector(sty, len) => match access_type {
                AccessType::Slice(range) => {
                    let slice_range = range.to_slice_range();
                    if slice_range.end > len {
                        Err(InvalidAccessError::IndexOutOfBounds)
                    } else {
                        Ok(Self::Vector(sty, slice_range.len()))
                    }
                },
                AccessType::Index(idx) if idx >= len => Err(InvalidAccessError::IndexOutOfBounds),
                AccessType::Index(_) => Ok(Self::Scalar(sty)),
                AccessType::Matrix(..) => Err(InvalidAccessError::IndexIntoScalar),
                _ => unreachable!(),
            },
            Self::Matrix(sty, rows, cols) => match access_type {
                AccessType::Slice(range) => {
                    let slice_range = range.to_slice_range();
                    if slice_range.end > rows {
                        Err(InvalidAccessError::IndexOutOfBounds)
                    } else {
                        Ok(Self::Matrix(sty, slice_range.len(), cols))
                    }
                },
                AccessType::Index(idx) if idx >= rows => Err(InvalidAccessError::IndexOutOfBounds),
                AccessType::Index(_) => Ok(Self::Vector(sty, cols)),
                AccessType::Matrix(row, col) if row >= rows || col >= cols => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                },
                AccessType::Matrix(..) => Ok(Self::Scalar(sty)),
                _ => unreachable!(),
            },
        }
    }
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Scalar(sty) => write!(f, "{}", sty),
            Self::Vector(sty, n) => write!(f, "{}[{}]", sty, n),
            Self::Matrix(sty, rows, cols) => write!(f, "{}[{}, {}]", sty, rows, cols),
        }
    }
}
impl Typed for Type {
    fn ty(&self) -> Option<Type> {
        Some(*self)
    }
}

/// Represents the type signature of a function
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionType {
    /// An evaluator function, which has no results, and has
    /// a complex type signature due to the nature of trace bindings
    Evaluator(Vec<TraceSegment>),
    /// A standard function with one or more inputs, and a result
    Function(Vec<Type>, Type),
}
impl FunctionType {
    pub fn result(&self) -> Option<Type> {
        match self {
            Self::Evaluator(_) => None,
            Self::Function(_, result) => Some(*result),
        }
    }
}
