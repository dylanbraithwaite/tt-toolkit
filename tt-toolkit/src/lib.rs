pub mod debruijn_indexed;
pub use debruijn_indexed::DeBruijnIndexed;
pub use ttt_derive::DeBruijnIndexed;

pub mod substitute;
pub use substitute::{SubstError, Substitute};
pub use ttt_derive::Substitute;

pub mod context;
pub use context::{Context, ListContext, PartialContext};

pub mod contextual_eq;
pub use contextual_eq::ContextualEq;

pub mod evaluate;
pub use evaluate::{EvalError, Evaluate};
pub use ttt_derive::Evaluate;

pub mod attribute;
pub use attribute::{BidirAttribute, CheckAttribute, SynthAttribute};
pub use ttt_derive::{BidirAttribute, SynthAttribute, CheckAttribute};

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

