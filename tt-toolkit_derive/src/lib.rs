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
decl_derive! { [Evaluate, attributes(eval_target, context_type, binding, evaluate_with, evaluate_pattern, metadata, var_name, eval_error_type)] =>
    /// Derives an implementation of the [ttt::Evaluate] trait.
    /// 
    /// The default behaviour sets the associated type [Target](ttt::Evaluate::Target) to `Self`, and will call [evaluate](ttt::Evaluate::evaluate) recursively on each field 
    /// and will reconstruct the current variant from the evaluated fields.
    /// Custom evaluation behaviour can be specified with attributes on variants.
    /// 
    /// # Evaluator patterns
    /// 
    /// The simplest way to specify custom evaluation logic is with the `#[evaluate_pattern {...}]` attribute.
    /// This attribute accepts a match arm which matches a list of evaluated fields, and yields the evaluated expression in the arm body.
    /// 
    /// **TODO**: This should accept multiple match arms
    /// 
    /// ```
    /// use ttt::Evaluate;
    /// 
    /// #[derive(Debug, Clone, PartialEq, Evaluate)]
    /// enum AddExpr {
    ///     Num(#[metadata] i32),
    ///     #[evaluate_pattern { 
    ///         (AddExpr::Num(lhs), AddExpr::Num(rhs)) => AddExpr::Num(lhs + rhs) 
    ///     }]
    ///     Add(Box<AddExpr>, Box<AddExpr>),
    /// }
    /// 
    /// use AddExpr::*;
    /// let expr = Add(
    ///     Box::new(Num(19)), 
    ///     Box::new(Add(
    ///         Box::new(Num(12)), 
    ///         Box::new(Num(11))
    ///     ))
    /// );
    /// 
    /// let evalled = expr.evaluate_closed(false);
    /// 
    /// assert_eq!(evalled, Ok(Num(42)));
    /// ```
    /// 
    /// ## Returning errors 
    /// Evaluate patterns can use early returns to yield errors. By default the error type is set to [ttt::EvalError], but this can be overridden with the `#[eval_error_type]` attribute
    /// 
    /// ```
    /// use ttt::Evaluate;
    /// 
    /// #[derive(Debug, PartialEq)]
    /// enum DivError {
    ///     DivideByZero,
    /// }
    /// 
    /// #[derive(Debug, Clone, PartialEq, Evaluate)]
    /// #[eval_error_type(DivError)]
    /// enum DivExpr {
    ///     Num(#[metadata] f32),
    ///     #[evaluate_pattern { (DivExpr::Num(lhs), DivExpr::Num(rhs)) => 
    ///         if rhs == 0.0 {
    ///             return Err(DivError::DivideByZero)
    ///         } else {
    ///             DivExpr::Num(lhs / rhs) 
    ///         }
    ///     }]
    ///     Div(Box<DivExpr>, Box<DivExpr>),
    /// } 
    /// ```
    /// 
    /// A more common usage pattern for this is to use the `?` operator to propagate errors from other function calls.
    /// 
    /// ```
    /// use ttt::{DeBruijnIndexed, Substitute, Evaluate};
    /// 
    /// #[derive(Clone, DeBruijnIndexed, Substitute, Evaluate, PartialEq, Debug)]
    /// enum LambdaExpr {
    ///     Var(#[var_index] usize),
    ///     Lambda(#[binding] Box<LambdaExpr>),
    ///     #[evaluate_pattern {
    ///         (LambdaExpr::Lambda(body), arg) => body.substitute(&arg, 0)?
    ///     }]
    ///     App(Box<LambdaExpr>, Box<LambdaExpr>),
    /// } 
    /// ```
    /// 
    /// In this example, the default error type [ttt::EvalError] implements `From<ttt::SubstError>`, so the `?` operator will convert any errors returned by `body.substitute(...)`
    /// and return them as an [EvalError](ttt::EvalError).
    /// 
    /// Evaluator patterns cannot access the evaluation context because they are intended for expressing short transformations on the syntax nodes.
    /// If you need to access the context you should specify an evaluator function, detailed in the next section.
    /// 
    /// # Evaluator Functions
    /// 
    /// If you require a larger code block to evaluate a variant, or you need access to the context variable, you can extract the evaluation logic into a separate function
    /// and specify it with the `#[evaluate_with(...)]` attribute.
    /// This attribute accepts as a parameter any expression which resolves to a function with the type 
    /// `(&Context, EvalledField1, EvalledField2, ...) -> Result<Target, Error>`.
    /// 
    /// ```
    /// use ttt::{DeBruijnIndexed, Substitute, Evaluate, Context};
    /// 
    /// #[derive(Clone, DeBruijnIndexed, Substitute, Evaluate, PartialEq, Debug)]
    /// enum LambdaExpr {
    ///     #[evaluate_with(lookup_var)]
    ///     Var(#[var_index] usize),
    ///     Lambda(#[binding] Box<LambdaExpr>),
    ///     #[evaluate_pattern {
    ///         (LambdaExpr::Lambda(body), arg) => body.substitute(&arg, 0)?
    ///     }]
    ///     App(Box<LambdaExpr>, Box<LambdaExpr>),
    /// } 
    /// 
    /// fn lookup_var(ctx: &<LambdaExpr as Evaluate>::Context, var_index: usize) -> Result<LambdaExpr, ttt::EvalError> {
    ///     match ctx.get(var_index).unwrap() {
    ///         Some(value) => Ok(value),
    ///         None => Ok(LambdaExpr::Var(var_index))
    ///     }
    /// }
    /// ```
    /// 
    /// The above example could be implemented automatically in a macro for any variant containing a variable, but we leave this to the user to allow flexibility in handling
    /// variable errors. 
    /// 
    /// # Evaluating into a different type 
    /// 
    /// ```
    /// use ttt::{DeBruijnIndexed, Substitute, Evaluate};
    /// 
    /// #[derive(Clone, DeBruijnIndexed, Substitute, PartialEq, Debug)]
    /// enum LambdaValue {
    ///     Var(#[var_index] usize),
    ///     Lambda(#[binding] Box<LambdaValue>)
    /// }
    /// 
    /// #[derive(Clone, DeBruijnIndexed, Substitute, Evaluate, PartialEq, Debug)]
    /// //#[eval_target(LambdaValue)]
    /// enum LambdaExpr {
    ///     Var(#[var_index] usize),
    ///     Lambda(#[binding] Box<LambdaExpr>),
    ///     //#[evaluate_pattern { _ => todo!() }]
    ///     App(Box<LambdaExpr>, Box<LambdaExpr>),
    /// } 
    /// ```
    /// 
    /// # Unwrapping Variants
    /// 
    /// ```
    /// enum Expr {
    /// }
    /// struct App {
    ///     fst: Box<Expr>,
    ///     snd: Box<Expr>,
    /// }
    /// 
    /// ```
    #[proc_macro_error]
    evaluate::derive
}
