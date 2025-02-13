use thiserror::Error;

use crate::{DeBruijnIndexed, PartialContext, SubstError, Context};

#[derive(Debug, Error, PartialEq)]
pub enum EvalError {
    #[error(transparent)]
    SubstError(#[from] SubstError),
}

pub trait Evaluate: Clone {
    type Target: DeBruijnIndexed;
    type Error;
    // type ContextEntry;
    // type Context: Context<Entry = Option<Self::ContextEntry>>;
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
        under_binders: bool
    ) -> Result<Self::Target, Self::Error> 
    {
        self.evaluate(&Self::Context::empty(), under_binders)
    }

    fn normalise_closed(
        &self, 
        under_binders: bool
    ) -> Result<Self, Self::Error> 
    where 
        Self::Target: Into<Self>
    {
        self.normalise(&Self::Context::empty(), under_binders)
    }

    // It isn't sufficient to ask that Target: From<ContextEntry>, because, for example in the
    // implementation for Box<T>, this requires Box<T::Target>: T::ContextEntry, which only
    // would hold if `From` was resolved transitively from:
    // Box<T::Target>: From<T::Target>, and
    // T::Target: From<T::ContextEntry>
    // fn from_context_entry(entry: Self::ContextEntry) -> Self::Target;
}

impl<T: Evaluate> Evaluate for Box<T> {
    type Target = T::Target;

    type Error = T::Error;

    // type ContextEntry = T::ContextEntry;
    type Context = T::Context;

    fn evaluate(
        &self,
        ctx: &Self::Context,
        under_binders: bool,
    ) -> Result<Self::Target, Self::Error> {
        (**self).evaluate(ctx, under_binders)
    }

    // fn from_context_entry(entry: Self::ContextEntry) -> Self::Target {
    //     Box::new(T::from_context_entry(entry))
    // }
}
