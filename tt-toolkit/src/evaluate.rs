use crate::Context;

pub trait Evaluate: Clone {
    type Target;
    type Error;
    // type ContextEntry;
    // type Context: Context<Entry = Option<Self::ContextEntry>>;
    type Context: Context<Entry = Option<Self::Target>>;

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
