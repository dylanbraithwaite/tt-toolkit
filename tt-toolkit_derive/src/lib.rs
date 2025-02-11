mod attributes;
mod debruijn_indexed;
mod utils;

use proc_macro_error::proc_macro_error;
use synstructure::decl_derive;

decl_derive! { [DeBruijnIndexed, attributes(var_index, variable, binding, metadata)] =>
    /// Derives an implementation of the `ttt::DeBruijnIndexed` trait.
    ///
    /// # Annotating variables
    ///
    /// The fields representing de Bruijn indices in a type should either:
    ///  - be a field of type `usize`, marked with the `#[var_index]` attribute
    ///  - or a type which wraps a de Bruijn index, marked with the `#[variable]` attribute
    ///
    /// ## Example
    /// ```
    /// use ttt::DeBruijnIndexed;
    ///
    /// #[derive(DeBruijnIndexed)]
    /// struct Variable(#[var_index] usize);
    ///
    ///
    /// #[derive(DeBruijnIndexed)]
    /// enum ContainsAVar {
    ///     HasNoVar,
    ///     HasAVar(#[variable] Variable),
    /// }
    /// ```
    ///
    /// # Binders
    ///
    /// Fields which represent syntax nodes under a binder should be annotated with the `#[binding]` attribute.
    /// These fields will be implemented with logic to shift target indices when recursively applying operations to their indices.
    ///
    /// ## Example
    /// ```
    /// use ttt::DeBruijnIndexed;
    ///
    /// #[derive(DeBruijnIndexed)]
    /// enum LambdaTerm {
    ///     Var(#[var_index] usize),
    ///     Lam(#[binding] Box<LambdaTerm>),
    ///     App(Box<LambdaTerm>, Box<LambdaTerm>),
    /// }
    ///
    /// #[derive(DeBruijnIndexed)]
    /// enum LambdaTermWithInnerVarType {
    ///     Var(#[variable] Variable),
    ///     Lam(#[binding] Box<LambdaTerm>),
    ///     App(Box<LambdaTerm>, Box<LambdaTerm>),
    /// }
    ///
    /// #[derive(DeBruijnIndexed)]
    /// struct Variable(#[var_index] usize);
    /// ```
    ///
    /// # Skipping metadata
    ///
    /// Fields which do not contain AST data can be marked with the `#[metadata]` attribute and they will be ignored when applying variable operations.
    /// This might be used for example if storing a string representation of a variable name alongside its de Bruijn index.
    ///
    /// ## Example
    /// ```
    /// #[derive(ttt::DeBruijnIndexed)]
    /// enum LambdaTerm {
    ///     Var {
    ///         #[metadata] name: String,
    ///         #[var_index] dbn_index: usize
    ///     },
    ///     Lam(#[binding] Box<LambdaTerm>),
    ///     App(Box<LambdaTerm>, Box<LambdaTerm>),
    /// }
    ///
    /// ```
    #[proc_macro_error]
    debruijn_indexed::derive
}
