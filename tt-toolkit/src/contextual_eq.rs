use crate::Context;

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

pub trait SyntacticEq: PartialEq {}

impl<Entry, Ctx, T> ContextualEq<Entry, Ctx> for T
where
    Ctx: Context<Entry>,
    T: SyntacticEq,
{
    type Check = bool;
    type Error = std::convert::Infallible;

    fn equiv(
        _ctx: &Ctx,
        lhs: &Self,
        rhs: &Self,
    ) -> Result<Self::Check, Self::Error> {
        Ok(*lhs == *rhs)
    }
}
