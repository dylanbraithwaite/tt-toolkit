Derives an implementation of the [ttt::Evaluate] trait.

The default behaviour sets the associated type [Target](ttt::Evaluate::Target) to `Self`, and will call [evaluate](ttt::Evaluate::evaluate) recursively on each field
and will reconstruct the current variant from the evaluated fields.
Custom evaluation behaviour can be specified with attributes on variants.

# Evaluator patterns

The simplest way to specify custom evaluation logic is with the `#[evaluate_pattern {...}]` attribute.
This attribute accepts a match arm which matches a list of evaluated fields, and yields the evaluated expression in the arm body.

**TODO**: This should accept multiple match arms

```rust
use ttt::Evaluate;

#[derive(Debug, Clone, PartialEq, Evaluate)]
enum AddExpr {
    Num(#[metadata] i32),
    #[evaluate_pattern {
        (AddExpr::Num(lhs), AddExpr::Num(rhs)) => AddExpr::Num(lhs + rhs)
    }]
    Add(Box<AddExpr>, Box<AddExpr>),
}

use AddExpr::*;
let expr = Add(
    Box::new(Num(19)),
    Box::new(Add(
        Box::new(Num(12)),
        Box::new(Num(11))
    ))
);

let evalled = expr.evaluate_closed(false);

assert_eq!(evalled, Ok(Num(42)));
```

## Returning errors
Evaluate patterns can use early returns to yield errors. By default the error type is set to [ttt::EvalError], but this can be overridden with the `#[eval_error_type]` attribute

```rust
use ttt::Evaluate;

#[derive(Debug, PartialEq)]
enum DivError {
    DivideByZero,
}

#[derive(Debug, Clone, PartialEq, Evaluate)]
#[eval_error_type(DivError)]
enum DivExpr {
    Num(#[metadata] f32),
    #[evaluate_pattern { (DivExpr::Num(lhs), DivExpr::Num(rhs)) =>
        if rhs == 0.0 {
            return Err(DivError::DivideByZero)
        } else {
            DivExpr::Num(lhs / rhs)
        }
    }]
    Div(Box<DivExpr>, Box<DivExpr>),
}
```

A more common usage pattern for this is to use the `?` operator to propagate errors from other function calls.

```rust
use ttt::{DeBruijnIndexed, Substitute, Evaluate};

#[derive(Clone, DeBruijnIndexed, Substitute, Evaluate, PartialEq, Debug)]
enum LambdaExpr {
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    #[evaluate_pattern {
        (LambdaExpr::Lambda(body), arg) => body.substitute(&arg, 0)?
    }]
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}
```

In this example, the default error type [ttt::EvalError] implements `From<ttt::SubstError>`, so the `?` operator will convert any errors returned by `body.substitute(...)`
and return them as an [EvalError](ttt::EvalError).

Evaluator patterns cannot access the evaluation context because they are intended for expressing short transformations on the syntax nodes.
If you need to access the context you should specify an evaluator function, detailed in the next section.

# Evaluator Functions

If you require a larger code block to evaluate a variant, or you need access to the context variable, you can extract the evaluation logic into a separate function
and specify it with the `#[evaluate_with(...)]` attribute.
This attribute accepts as a parameter any expression which resolves to a function with the type
`(&Context, EvalledField1, EvalledField2, ...) -> Result<Target, Error>`.

```rust
/*
TODO: Compile error
use ttt::{DeBruijnIndexed, Substitute, Evaluate, Context};

#[derive(Clone, DeBruijnIndexed, Substitute, Evaluate, PartialEq, Debug)]
enum LambdaExpr {
    #[evaluate_with(lookup_var)]
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    #[evaluate_pattern {
        (LambdaExpr::Lambda(body), arg) => body.substitute(&arg, 0)?
    }]
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}

fn lookup_var(ctx: &<LambdaExpr as Evaluate>::Context, var_index: usize) -> Result<LambdaExpr, ttt::EvalError> {
    match ctx.get(var_index).unwrap() {
        Some(value) => Ok(value),
        None => Ok(LambdaExpr::Var(var_index))
    }
}
*/
```

The above example could be implemented automatically in a macro for any variant containing a variable, but we leave this to the user to allow flexibility in handling
variable errors.

# Evaluating into a different type

```rust
use ttt::{DeBruijnIndexed, Substitute, Evaluate};

#[derive(Clone, DeBruijnIndexed, Substitute, PartialEq, Debug)]
enum LambdaValue {
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaValue>)
}

#[derive(Clone, DeBruijnIndexed, Substitute, Evaluate, PartialEq, Debug)]
//#[eval_target(LambdaValue)]
enum LambdaExpr {
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    //#[evaluate_pattern { _ => todo!() }]
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}
```

# Unwrapping Variants

```rust
enum Expr {
}
struct App {
    fst: Box<Expr>,
    snd: Box<Expr>,
}

```