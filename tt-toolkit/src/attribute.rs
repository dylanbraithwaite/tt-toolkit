use crate::{Context, ContextualEq};

/// Represents a syntax type which can synthesize an attribute of type `Attr` from its syntax tree.
pub trait SynthAttribute<Attr> {
    type Error;
    type Entry;
    type Ctx: Context<Self::Entry>;
    fn synth(&self, ctx: &Self::Ctx) -> Result<Attr, Self::Error>;
}

pub trait CheckAttribute<Attr> {
    type Check;
    type Error;
    type Entry;
    type Ctx: Context<Self::Entry>;

    fn check(
        &self,
        ctx: &Self::Ctx,
        attr: &Attr,
    ) -> Result<Self::Check, Self::Error>;
}

impl<Attr, Expr: SynthAttribute<Attr>> SynthAttribute<Option<Attr>> for Expr {
    type Error = Expr::Error;

    type Entry = Expr::Entry;

    type Ctx = Expr::Ctx;

    fn synth(&self, ctx: &Self::Ctx) -> Result<Option<Attr>, Self::Error> {
        Ok(Some(self.synth(ctx)?))
    }
}

pub trait BidirAttribute<Attr>:
    CheckAttribute<Attr>
    + SynthAttribute<
        Option<Attr>,
        Error = <Self as CheckAttribute<Attr>>::Error,
        Entry = <Self as CheckAttribute<Attr>>::Entry,
        Ctx = <Self as CheckAttribute<Attr>>::Ctx,
    >
{
}

impl<Expr: SynthAttribute<Attr>, Attr> CheckAttribute<Attr> for Expr
where
    Attr: ContextualEq<Expr::Entry, Expr::Ctx>,
    Expr::Error: From<Attr::Error>,
{
    type Check = Attr::Check;

    type Error = Expr::Error;
    type Entry = Expr::Entry;
    type Ctx = Expr::Ctx;

    fn check(
        &self,
        ctx: &Self::Ctx,
        attr: &Attr,
    ) -> Result<Self::Check, Self::Error> {
        Ok(Attr::equiv(ctx, &self.synth(ctx)?, attr)?)
    }
}

impl<Expr: SynthAttribute<Attr>, Attr> BidirAttribute<Attr> for Expr
where
    Attr: ContextualEq<Expr::Entry, Expr::Ctx>,
    Expr::Error: From<Attr::Error>,
{
}
