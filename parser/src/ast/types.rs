use super::*;

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ScalarType {
    /// A field element
    Felt,
    /// An integer
    Int,
    /// A binary number
    Bool,
    /// An untyped value, used for type inference
    #[default]
    Untyped,
}

impl PartialOrd for ScalarType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (Self::Untyped, _) | (_, Self::Untyped) => std::cmp::Ordering::Equal,
            (Self::Felt, Self::Felt) | (Self::Int, Self::Int) | (Self::Bool, Self::Bool) => {
                std::cmp::Ordering::Equal
            }
            (Self::Felt, _) => std::cmp::Ordering::Greater,
            (_, Self::Felt) => std::cmp::Ordering::Less,
            (Self::Int, Self::Bool) => std::cmp::Ordering::Greater,
            (Self::Bool, Self::Int) => std::cmp::Ordering::Less,
        })
    }
}
impl Ord for ScalarType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Felt => write!(f, "felt"),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Untyped => write!(f, "_"),
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
            Self::Vector(_, _) | Self::Matrix(_, _, _) => true,
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
        matches!(self, Self::Vector(_, _))
    }

    /// Return a new [Type] representing the type of the value produced by the given [AccessType]
    pub fn access(&self, access_type: AccessType) -> Result<Self, InvalidAccessError> {
        match *self {
            ty if access_type == AccessType::Default => Ok(ty),
            Self::Scalar(_) => Err(InvalidAccessError::IndexIntoScalar),
            Self::Vector(ty, len) => match access_type {
                AccessType::Slice(range) => {
                    let slice_range = range.to_slice_range();
                    if slice_range.end > len {
                        Err(InvalidAccessError::IndexOutOfBounds)
                    } else {
                        Ok(Self::Vector(ty, slice_range.len()))
                    }
                }
                AccessType::Index(idx) if idx >= len => Err(InvalidAccessError::IndexOutOfBounds),
                AccessType::Index(_) => Ok(Self::Scalar(ty)),
                AccessType::Matrix(_, _) => Err(InvalidAccessError::IndexIntoScalar),
                _ => unreachable!(),
            },
            Self::Matrix(ty, rows, cols) => match access_type {
                AccessType::Slice(range) => {
                    let slice_range = range.to_slice_range();
                    if slice_range.end > rows {
                        Err(InvalidAccessError::IndexOutOfBounds)
                    } else {
                        Ok(Self::Matrix(ty, slice_range.len(), cols))
                    }
                }
                AccessType::Index(idx) if idx >= rows => Err(InvalidAccessError::IndexOutOfBounds),
                AccessType::Index(_) => Ok(Self::Vector(ty, cols)),
                AccessType::Matrix(row, col) if row >= rows || col >= cols => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                }
                AccessType::Matrix(_, _) => Ok(Self::Scalar(ty)),
                _ => unreachable!(),
            },
        }
    }
    pub fn scalar_type(&self) -> ScalarType {
        match self {
            Self::Scalar(sty) => *sty,
            Self::Vector(sty, _) => *sty,
            Self::Matrix(sty, _, _) => *sty,
        }
    }
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Scalar(ty) => write!(f, "{}", ty),
            Self::Vector(ty, n) => write!(f, "{}[{}]", ty, n),
            Self::Matrix(ty, rows, cols) => write!(f, "{}[{}, {}]", ty, rows, cols),
        }
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
