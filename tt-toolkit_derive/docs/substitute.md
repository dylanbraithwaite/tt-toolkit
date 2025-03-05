Derives implementations of the `ttt::Substitute` trait.

The macro has two modes of operation:
 - the typical case is 'deep substitution', replacing a variable inside a data type which an expression of the same type,
 - alternatively, if the type is a *variable wrapper* (meaning a struct containing a variable, or an enum where every variant contains a variable), then we implement 'shallow substitution' which will replace the entire structure with an expression, or embed the variable into the target type.

# Deep Substitution

In the typical case, a type `T` deriving `Substitute` will implement `Substitute<T, Target=T>`.

## Example
```rust
use ttt::{DeBruijnIndexed, Substitute};

// LambdaExpr: Substitute<LambdaExpr, Target = LambdaExpr>
#[derive(Clone, DeBruijnIndexed, Substitute)]
enum LambdaExpr {
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}
```

## Substituting for multiple types

A list of types to be substituted for may be specified as an annotation on the deriving type, as `#[subst_types(T, U, ...)]`.
In this case `Substitute<T>` will be implemented for each type `T` specified in the list.
This may be useful if your type contains an inner type with its own substitute implementation which operates on different types.

### Example
```rust
use ttt::{DeBruijnIndexed, Substitute};

// LambdaValue: Substitute<LambdaValue, Target = LambdaValue>
// LambdaExpr: Substitute<LambdaValue, Target = LambdaExpr>
// LambdaExpr: Substitute<LambdaExpr, Target = LambdaExpr>

#[derive(Clone, DeBruijnIndexed, Substitute)]
#[subst_types(LambdaExpr, LambdaValue)]
enum LambdaValue {
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaValue>),
}

#[derive(Clone, DeBruijnIndexed, Substitute)]
#[subst_types(LambdaExpr, LambdaValue)]
enum LambdaExpr {
    Value(LambdaValue),
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}
```

# Shallow substitution

A variable is any field annotated with the `#[variable]` attribute, or any field of type `usize` which is annotated with `#[var_index]`.
A variable wrapper is any struct containing a variable, or an enum where every variant contains a variable.
If a type `Var` is a variable wrapper, then `Substitute<Expr, Target=Expr>` can be derived on `Var` for any type `Expr`, as long as `Expr: From<Var>`.

### Example
```rust
use ttt::{DeBruijnIndexed, Substitute};

#[derive(PartialEq, Debug, Clone, DeBruijnIndexed, Substitute)]
enum LambdaExpr {
    Var(#[variable] Variable),
    Lambda(#[binding] Box<LambdaExpr>),
    App(Box<LambdaExpr>, Box<LambdaExpr>),
    Unit,
}

// Variable: Substitute<LambdaExpr>
#[derive(PartialEq, Debug, Clone, DeBruijnIndexed, Substitute)]
#[subst_types(LambdaExpr)]
struct Variable {
    #[var_index] index: usize,
    #[metadata] name: String
}

impl From<Variable> for LambdaExpr {
    fn from(value: Variable) -> Self {
        LambdaExpr::Var(value)
    }
}

let variable = Variable {
    index: 0,
    name: "".to_string(),
};

let lambda_expr = LambdaExpr::Lambda( Box::new(LambdaExpr::Unit) );

let substituted: Result<LambdaExpr, _> = variable.substitute(&lambda_expr, 0);
assert_eq!(substituted, Ok(lambda_expr.clone()));

// No substitution occurs because the variables do not match, but
// `variable` is wrapped in a `LambdaExpr::Var(..)` so that it has
// the correct type.
let substituted2 = variable.substitute(&lambda_expr, 1);
assert_eq!(substituted2, Ok(LambdaExpr::Var(variable)));
```