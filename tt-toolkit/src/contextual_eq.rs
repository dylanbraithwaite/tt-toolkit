use std::marker::PhantomData;

use crate::{Context, Evaluate, Never};

pub trait ContextualEq<Entry, Ctx>
where
    Ctx: Context<Entry>,
{
    type Check;
    type Error;

    fn equiv(
        ctx: &Ctx,
        lhs: &Self,
        rhs: &Self,
    ) -> Result<Self::Check, Self::Error>;
}

pub trait AutoContextualEqImpl<Entry, Ctx: Context<Entry>, T: ?Sized> {
    type Check;
    type Error;

    fn equiv(ctx: &Ctx, lhs: &T, rhs: &T) -> Result<Self::Check, Self::Error>;
}

pub struct SyntacticEq<T: PartialEq>(PhantomData<T>);

pub struct NormalFormEq<T>(PhantomData<T>)
where
    T: Evaluate,
    T::Target: PartialEq;

impl<Entry, Ctx: Context<Entry>, T: PartialEq>
    AutoContextualEqImpl<Entry, Ctx, T> for SyntacticEq<T>
{
    type Check = bool;
    type Error = Never;

    fn equiv(_ctx: &Ctx, lhs: &T, rhs: &T) -> Result<Self::Check, Self::Error> {
        Ok(*lhs == *rhs)
    }
}

impl<T: Evaluate> AutoContextualEqImpl<Option<T::Target>, T::Context, T>
    for NormalFormEq<T>
where
    T::Target: PartialEq,
{
    type Check = bool;

    type Error = T::Error;

    fn equiv(
        ctx: &T::Context,
        lhs: &T,
        rhs: &T,
    ) -> Result<Self::Check, Self::Error> {
        Ok(lhs.evaluate(ctx, true)? == rhs.evaluate(ctx, true)?)
    }
}

pub trait AutoContextualEq<Entry, Ctx: Context<Entry>> {
    type Impl: AutoContextualEqImpl<Entry, Ctx, Self>;
}

impl<Entry, Ctx: Context<Entry>, T> ContextualEq<Entry, Ctx> for T
where
    T: AutoContextualEq<Entry, Ctx>,
{
    type Check = <T::Impl as AutoContextualEqImpl<Entry, Ctx, T>>::Check;
    type Error = <T::Impl as AutoContextualEqImpl<Entry, Ctx, T>>::Error;

    fn equiv(
        ctx: &Ctx,
        lhs: &Self,
        rhs: &Self,
    ) -> Result<Self::Check, Self::Error> {
        T::Impl::equiv(ctx, lhs, rhs)
    }
}

// /// Marker for syntax nodes whose PartialEq implementation provides a meaningful notion of semantic equivalence.
// pub trait SyntacticEq: PartialEq {}

// pub trait NormalFormEq: Evaluate
// where
//     Self::Target: PartialEq,
// {
// }

// impl<Entry, Ctx, T> ContextualEq<Entry, Ctx> for T
// where
//     Ctx: Context<Entry>,
//     T: SyntacticEq,
// {
//     type Check = bool;
//     type Error = Never;

//     fn equiv(
//         _ctx: &Ctx,
//         lhs: &Self,
//         rhs: &Self,
//     ) -> Result<Self::Check, Self::Error> {
//         Ok(*lhs == *rhs)
//     }
// }
