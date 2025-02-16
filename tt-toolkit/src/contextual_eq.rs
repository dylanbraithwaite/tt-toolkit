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

impl<Entry, Ctx: Context<Entry>, T: SyntacticEq> ContextualEq<Entry, Ctx>
    for T
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
