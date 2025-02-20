use thiserror::Error;

use crate::{Context, PartialContext, SubstError};

#[derive(Debug, Error, PartialEq)]
pub enum EvalError {
    #[error(transparent)]
    SubstError(#[from] SubstError),
}

pub trait Evaluate: Clone {
    type Target;
    type Error;
    type Context: PartialContext<Self::Target>;

    fn evaluate(
        &self,
        ctx: &Self::Context,
        under_binders: bool,
    ) -> Result<Self::Target, Self::Error>;

    fn normalise(
        &self,
        ctx: &Self::Context,
        under_binders: bool,
    ) -> Result<Self, Self::Error>
    where
        Self::Target: Into<Self>,
    {
        self.evaluate(ctx, under_binders).map(Into::into)
    }

    fn evaluate_closed(
        &self,
        under_binders: bool,
    ) -> Result<Self::Target, Self::Error> {
        self.evaluate(&Self::Context::empty(), under_binders)
    }

    fn normalise_closed(&self, under_binders: bool) -> Result<Self, Self::Error>
    where
        Self::Target: Into<Self>,
    {
        self.normalise(&Self::Context::empty(), under_binders)
    }
}

impl<T: Evaluate> Evaluate for Box<T> {
    type Target = T::Target;

    type Error = T::Error;

    type Context = T::Context;

    fn evaluate(
        &self,
        ctx: &Self::Context,
        under_binders: bool,
    ) -> Result<Self::Target, Self::Error> {
        (**self).evaluate(ctx, under_binders)
    }
}
