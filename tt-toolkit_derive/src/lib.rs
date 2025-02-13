mod attributes;
mod utils;

use proc_macro_error::proc_macro_error;
use synstructure::decl_derive;

mod debruijn_indexed;
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
    /// For the purposes of this macro, `#[var_name]` and `#[binding_name]` have equivalent effects to `#[metadata]`, but these have more specific effects in other macros.
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

mod substitute;
decl_derive! { [Substitute, attributes(var_index, subst_types, variable, binding)] =>
    /// Derives implementations of the `ttt::Substitute` trait.
    ///
    /// The macro has two modes of operation:
    ///  - the typical case is 'deep substitution', replacing a variable inside a data type which an expression of the same type,
    ///  - alternatively, if the type is a *variable wrapper* (meaning a struct containing a variable, or an enum where every variant contains a variable), then we implement 'shallow substitution' which will replace the entire structure with an expression, or embed the variable into the target type.
    ///
    /// # Deep Substitution
    ///
    /// In the typical case, a type `T` deriving `Substitute` will implement `Substitute<T, Target=T>`.
    ///
    /// ## Example
    /// ```
    /// use ttt::{DeBruijnIndexed, Substitute};
    ///
    /// // LambdaExpr: Substitute<LambdaExpr, Target = LambdaExpr>
    /// #[derive(Clone, DeBruijnIndexed, Substitute)]
    /// enum LambdaExpr {
    ///     Var(#[var_index] usize),
    ///     Lambda(#[binding] Box<LambdaExpr>),
    ///     App(Box<LambdaExpr>, Box<LambdaExpr>),
    /// }
    /// ```
    ///
    /// ## Substituting for multiple types
    ///
    /// A list of types to be substituted for may be specified as an annotation on the deriving type, as `#[subst_types(T, U, ...)]`.
    /// In this case `Substitute<T>` will be implemented for each type `T` specified in the list.
    /// This may be useful if your type contains an inner type with its own substitute implementation which operates on different types.
    ///
    /// ### Example
    /// ```
    /// use ttt::{DeBruijnIndexed, Substitute};
    ///
    /// // LambdaValue: Substitute<LambdaValue, Target = LambdaValue>
    /// // LambdaExpr: Substitute<LambdaValue, Target = LambdaExpr>
    /// // LambdaExpr: Substitute<LambdaExpr, Target = LambdaExpr>
    ///
    /// #[derive(Clone, DeBruijnIndexed, Substitute)]
    /// #[subst_types(LambdaExpr, LambdaValue)]
    /// enum LambdaValue {
    ///     Var(#[var_index] usize),
    ///     Lambda(#[binding] Box<LambdaValue>),
    /// }
    ///
    /// #[derive(Clone, DeBruijnIndexed, Substitute)]
    /// #[subst_types(LambdaExpr, LambdaValue)]
    /// enum LambdaExpr {
    ///     Value(LambdaValue),
    ///     Var(#[var_index] usize),
    ///     Lambda(#[binding] Box<LambdaExpr>),
    ///     App(Box<LambdaExpr>, Box<LambdaExpr>),
    /// }
    /// ```
    ///
    /// # Shallow substitution
    ///
    /// A variable is any field annotated with the `#[variable]` attribute, or any field of type `usize` which is annotated with `#[var_index]`.
    /// A variable wrapper is any struct containing a variable, or an enum where every variant contains a variable.
    /// If a type `Var` is a variable wrapper, then `Substitute<Expr, Target=Expr>` can be derived on `Var` for any type `Expr`, as long as `Expr: From<Var>`.
    ///
    /// ### Example
    /// ```
    /// use ttt::{DeBruijnIndexed, Substitute};
    ///
    /// #[derive(PartialEq, Debug, Clone, DeBruijnIndexed, Substitute)]
    /// enum LambdaExpr {
    ///     Var(#[variable] Variable),
    ///     Lambda(#[binding] Box<LambdaExpr>),
    ///     App(Box<LambdaExpr>, Box<LambdaExpr>),
    ///     Unit,
    /// }
    ///
    /// // Variable: Substitute<LambdaExpr>
    /// #[derive(PartialEq, Debug, Clone, DeBruijnIndexed, Substitute)]
    /// #[subst_types(LambdaExpr)]
    /// struct Variable {
    ///     #[var_index] index: usize,
    ///     #[metadata] name: String
    /// }
    ///
    /// impl From<Variable> for LambdaExpr {
    ///     fn from(value: Variable) -> Self {
    ///         LambdaExpr::Var(value)
    ///     }
    /// }
    ///
    /// let variable = Variable {
    ///     index: 0,
    ///     name: "".to_string(),
    /// };
    ///
    /// let lambda_expr = LambdaExpr::Lambda( Box::new(LambdaExpr::Unit) );
    ///
    /// let substituted: Result<LambdaExpr, _> = variable.substitute(&lambda_expr, 0);
    /// assert_eq!(substituted, Ok(lambda_expr.clone()));
    ///
    /// // No substitution occurs because the variables do not match, but
    /// // `variable` is wrapped in a `LambdaExpr::Var(..)` so that it has
    /// // the correct type.
    /// let substituted2 = variable.substitute(&lambda_expr, 1);
    /// assert_eq!(substituted2, Ok(LambdaExpr::Var(variable)));
    /// ```
    #[proc_macro_error]
    substitute::derive
}

mod evaluate;
decl_derive! { [Evaluate, attributes(eval_target, context_type, binding, evaluate_with, evaluate_pattern, metadata, var_name)] =>
    #[proc_macro_error]
    evaluate::derive
}
