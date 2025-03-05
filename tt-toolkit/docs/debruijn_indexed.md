Derives an implementation of the [`trait@DeBruijnIndexed`] trait.

# Annotating variables

The fields representing de Bruijn indices in a type should either:
 - be a field of type `usize`, marked with the `#[var_index]` attribute
 - or a type which wraps a de Bruijn index, marked with the `#[variable]` attribute

## Example
```rust
use ttt::DeBruijnIndexed;

#[derive(DeBruijnIndexed)]
struct Variable(#[var_index] usize);


#[derive(DeBruijnIndexed)]
enum ContainsAVar {
    HasNoVar,
    HasAVar(#[variable] Variable),
}
```

# Binders

Fields which represent syntax nodes under a binder should be annotated with the `#[binding]` attribute.
These fields will be implemented with logic to shift target indices when recursively applying operations to their indices.

## Example
```rust
use ttt::DeBruijnIndexed;

#[derive(DeBruijnIndexed)]
enum LambdaTerm {
    Var(#[var_index] usize),
    Lam(#[binding] Box<LambdaTerm>),
    App(Box<LambdaTerm>, Box<LambdaTerm>),
}

#[derive(DeBruijnIndexed)]
enum LambdaTermWithInnerVarType {
    Var(#[variable] Variable),
    Lam(#[binding] Box<LambdaTerm>),
    App(Box<LambdaTerm>, Box<LambdaTerm>),
}

#[derive(DeBruijnIndexed)]
struct Variable(#[var_index] usize);
```

# Skipping metadata

Fields which do not contain AST data can be marked with the `#[metadata]` attribute and they will be ignored when applying variable operations.
This might be used for example if storing a string representation of a variable name alongside its de Bruijn index.
For the purposes of this macro, `#[var_name]` and `#[binding_name]` have equivalent effects to `#[metadata]`, but these have more specific effects in other macros.

## Example
```rust
#[derive(ttt::DeBruijnIndexed)]
enum LambdaTerm {
    Var {
        #[metadata] name: String,
        #[var_index] dbn_index: usize
    },
    Lam(#[binding] Box<LambdaTerm>),
    App(Box<LambdaTerm>, Box<LambdaTerm>),
}

```