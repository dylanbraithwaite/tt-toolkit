pub mod debruijn_indexed;
#[doc(inline)]
pub use debruijn_indexed::DeBruijnIndexed;
#[doc = include_str!("../docs/debruijn_indexed.md")]
#[doc(inline)]
pub use ttt_derive::DeBruijnIndexed;

pub mod substitute;
#[doc(inline)]
pub use substitute::{SubstError, Substitute};
#[doc = include_str!("../docs/substitute.md")]
#[doc(inline)]
pub use ttt_derive::Substitute;

pub mod context;
#[doc(inline)]
pub use context::{Context, ListContext, PartialContext};

pub mod contextual_eq;
#[doc(inline)]
pub use contextual_eq::ContextualEq;

pub mod evaluate;
#[doc(inline)]
pub use evaluate::{EvalError, Evaluate};
#[doc = include_str!("../docs/evaluate.md")]
#[doc(inline)]
pub use ttt_derive::Evaluate;

pub mod attribute;
#[doc(inline)]
pub use attribute::{BidirAttribute, CheckAttribute, SynthAttribute};
#[doc = include_str!("../docs/attributed.md")]
#[doc(inline)]
pub use ttt_derive::Attributed;

#[doc(hidden)]
pub use ::spez;

mod never {
    pub
    trait FnOnce<Args> {
        type Output;
    }

    impl<F, R> FnOnce<()> for F
    where
        F : ::core::ops::FnOnce() -> R,
    {
        type Output = R;
    }

    pub type Never = <fn()->! as FnOnce<()>>::Output;
}

pub use never::Never;


#[derive(Clone, PartialEq, Debug)]
pub struct DefaultError;

impl From<Never> for DefaultError {
    fn from(value: Never) -> Self {
        value
    }
}

