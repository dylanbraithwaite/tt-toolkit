pub mod debruijn_indexed;
pub use debruijn_indexed::DeBruijnIndexed;
pub use ttt_derive::DeBruijnIndexed;

pub mod substitute;
pub use substitute::{SubstError, Substitute};
pub use ttt_derive::Substitute;

pub mod context;
pub use context::{Context, ListContext, PartialContext};

pub mod evaluate;
pub use evaluate::{Evaluate, EvalError};
pub use ttt_derive::Evaluate;
