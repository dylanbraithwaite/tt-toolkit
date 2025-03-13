Derives implementations of the [`trait@CheckAttribute`], [`trait@SynthAttribute`], and [`trait@BidirAttribute`] traits, using a DSL
annotated on variants or type definitions.

Any type deriving `Attributed` should specify at least one of the `synth_type`, `check_type`, or `bidir_type` attributes, whose usage is detailed in the following section.

## Example Usage
```rust
use ttt::*;

#[derive(PartialEq, Clone, Debug)]
enum Ty {
    Unit,
    Func(Box<Ty>, Box<Ty>),
}

impl contextual_eq::SyntacticEq for Ty {}

#[derive(Attributed)]
#[synth_type(Ty)]
enum Expr {
    #[synth(Ty; () => Ty::Unit)]
    Unit,

    #[synth(Ty; (src_ty, body) =>
        let tgt_ty = bind src_ty { synth(body) };
        Ty::Func(src_ty.clone().into(), tgt_ty.into())
    )]
    Lamba(Ty, Box<Expr>),

    #[synth(Ty; (func, arg) =>
        let Ty::Func(src, tgt) = synth(func);
        check(arg, src.as_ref());
        *tgt
    )]
    App(Box<Expr>, Box<Expr>),
}
```

# Specifying attribute types
The types of attribute which are supported by a syntax node can be specified by annotating the node type with the following attributes, where `Attr` is the type representing the attribute in question:
 - `#[check_type(Attr)]`
   - For attributes which may not be computable, but whose validity can be checked for a specific instance.
   - Provides an implementation of [`trait@CheckAttribute<Attr>`].
 - `#[synth_type(Attr)]`
   - Indicating that the attributes can always be synthesised from any well-formed instance of the syntax node
   - Provides an implementation of [`trait@SynthAttribute<Attr>`].
   - If `Attr` implements [`trait@ContextualEq`] then a blanket implementation of [`trait@CheckAttribute<Attr>`] will be available.
 - `#[bidir_type(Attr)]` 
   - For attributes which are specified as a combination of synthesis and checking rules on variants of an enum.
   - Provides an implementation of [`trait@CheckAttribute<Attr>`], [`trait@SynthAttribute<Option<Attr>>`], and [`trait@BidirAttribute<Attr>`].

Additional options may be provided in these attributes, as a comma separated list of `key = value` pairs, to customise the implementation of the corresponding traits.

## Context entries
By default derived implementations use a context implementing [`trait@Context<Attr>`].
You can customise this behaviour to use a type other than `Attr` (for example if you need to store additional information alongside attribute values) using the `context_entry` key; i.e. as in `#[synth_type(Attr, context_entry = Entry)]`

## Custom context types
An implementation can further provide a concrete context type using the `context = ContextType` option. 
The default behaviour for this option is as if the type was specified as `#[synth_type(Attr, context_entry = Entry, context = ttt::ListContext<Entry>)]`

## Normalising subterms

# The attribute DSL

## Synth expressions

## Check expressions

## Lookup expressions

## Fallible matching

## Bind expressions

## Returning errors

# Synth attributes

# Check attributes

# Bidirectional attributes
