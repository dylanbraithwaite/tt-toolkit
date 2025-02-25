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
        attr: &Attr,
        ctx: &Self::Ctx,
    ) -> Result<Self::Check, Self::Error>;
}

#[diagnostic::do_not_recommend]
impl<Attr, Expr> SynthAttribute<Option<Attr>> for Expr
where
    Expr: SynthAttribute<Attr>,
{
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

#[diagnostic::do_not_recommend]
impl<Expr, Attr> CheckAttribute<Attr> for Expr
where
    Expr: SynthAttribute<Attr>,
    Attr: ContextualEq<Expr::Entry, Expr::Ctx>,
    Expr::Error: From<Attr::Error>,
{
    type Check = Attr::Check;

    type Error = Expr::Error;
    type Entry = Expr::Entry;
    type Ctx = Expr::Ctx;

    fn check(
        &self,
        attr: &Attr,
        ctx: &Self::Ctx,
    ) -> Result<Self::Check, Self::Error> {
        Ok(Attr::equiv(ctx, &self.synth(ctx)?, attr)?)
    }
}

impl<Expr, Attr> BidirAttribute<Attr> for Expr
where
    Attr: ContextualEq<Expr::Entry, Expr::Ctx>,
    Expr: SynthAttribute<Attr>,
    Expr::Error: From<Attr::Error>,
{
}
