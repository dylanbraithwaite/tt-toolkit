use std::any::{TypeId, type_name};

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
#[error(
    "Tried to substitute a variable of kind {} for an expression of kind {}",
    substitutand_type_name,
    substitutee_type_name
)]
pub struct SubstError {
    substitutee_type: TypeId,
    substitutand_type: TypeId,
    substitutee_type_name: &'static str,
    substitutand_type_name: &'static str,
}

impl SubstError {
    pub fn new<T, U>() -> Self
    where
        T: 'static,
        U: 'static,
    {
        Self {
            substitutee_type: TypeId::of::<T>(),
            substitutand_type: TypeId::of::<U>(),
            substitutee_type_name: type_name::<T>(),
            substitutand_type_name: type_name::<U>(),
        }
    }
}

/// A syntax node which admits a substitution operation.
pub trait Substitute<SubstExpr> {
    /// The result of substituting an expression for a variable in this term. For
    /// most AST types this will be `Self`.
    type Target;

    /// The type of errors which may occur during substitution.
    /// In the simple case where `SubstExpr == Target` this should usually be set to
    /// `std::convert::Infallible`.
    /// However, in more complex cases you may want to implement `Substitute<E>` for other values of `E` (such as where you have mutually recursive expression types, each which may contain variables).
    /// In such cases `substitute` must handle the case where the variable index refers to a position in the wrong type of expression.
    /// N.B. currently the derive macro for `Substitute` will always use `SubstError` as the error type.
    type Error;

    /// Substitute `expr` into any variables in the expression matching the specified
    /// de Bruijn index.
    fn substitute(
        self,
        expr: SubstExpr,
        var: usize,
    ) -> Result<Self::Target, Self::Error>;
}

impl<T, U> Substitute<U> for Box<T>
where
    T: Substitute<U>,
{
    type Target = Box<T::Target>;
    type Error = T::Error;

    fn substitute(
        self,
        expr: U,
        var: usize,
    ) -> Result<Self::Target, Self::Error> {
        Ok(Box::new((*self).substitute(expr, var)?))
    }
}
