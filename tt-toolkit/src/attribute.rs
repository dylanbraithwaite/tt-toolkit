use crate::{Context, ContextualEq};

/// Represents a syntax type which can synthesize an attribute of type `Attr` from its syntax tree.
pub trait SynthAttribute<Attr> {
    type Error;
    type Entry;
    type Ctx: Context<Self::Entry>;

    fn synth(&self, ctx: &Self::Ctx) -> Result<Attr, Self::Error>;

    fn synth_closed(&self) -> Result<Attr, Self::Error> {
        self.synth(&Context::empty())
    }
}

pub trait PartialSynthAttribute<Attr> {
    type Error;
    type Entry;
    type Ctx: Context<Self::Entry>;

    fn try_synth(&self, ctx: &Self::Ctx) -> Result<Option<Attr>, Self::Error>;

    fn try_synth_closed(&self) -> Result<Option<Attr>, Self::Error> {
        self.try_synth(&Context::empty())
    }
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

    fn check_closed(&self, attr: &Attr) -> Result<Self::Check, Self::Error> {
        self.check(attr, &Context::empty())
    }
}

#[diagnostic::do_not_recommend]
impl<Attr, Expr> PartialSynthAttribute<Attr> for Expr
where
    Expr: SynthAttribute<Attr>,
{
    type Error = Expr::Error;

    type Entry = Expr::Entry;

    type Ctx = Expr::Ctx;

    fn try_synth(&self, ctx: &Self::Ctx) -> Result<Option<Attr>, Self::Error> {
        Ok(Some(self.synth(ctx)?))
    }
}

pub trait BidirAttribute<Attr>:
    CheckAttribute<
        Attr,
        Error = <Self as BidirAttribute<Attr>>::Error,
        Entry = <Self as BidirAttribute<Attr>>::Entry,
        Ctx = <Self as BidirAttribute<Attr>>::Ctx,
    > + PartialSynthAttribute<
        Attr,
        Error = <Self as BidirAttribute<Attr>>::Error,
        Entry = <Self as BidirAttribute<Attr>>::Entry,
        Ctx = <Self as BidirAttribute<Attr>>::Ctx,
    >
{
    type Error;
    type Entry;
    type Ctx;
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

#[diagnostic::do_not_recommend]
impl<Expr, Attr> BidirAttribute<Attr> for Expr
where
    Attr: ContextualEq<Expr::Entry, Expr::Ctx>,
    Expr: SynthAttribute<Attr>,
    Expr::Error: From<Attr::Error>,
{
    type Error = Expr::Error;
    type Entry = Expr::Entry;
    type Ctx = Expr::Ctx;
}
