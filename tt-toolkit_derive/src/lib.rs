//! Derive macros for the [tt-toolkit](https://crates.io/crates/tt-toolkit) crate.
//!
//! Documentation for the macros defined here can be found in the docs for the main crate.

mod attributes;
mod utils;

use proc_macro_error2::proc_macro_error;
use synstructure::decl_derive;

mod debruijn_indexed;
decl_derive! { [DeBruijnIndexed, attributes(var_index, variable, binding, metadata, var_name)] =>
    #[proc_macro_error]
    debruijn_indexed::derive
}

mod substitute;
decl_derive! { [Substitute, attributes(var_index, subst_types, variable, binding, inherit_subst_types)] =>
    #[proc_macro_error]
    substitute::derive
}

mod evaluate;
decl_derive! { [Evaluate, attributes(eval_target, context_type, binding, evaluate_with, evaluate_pattern, evaluate_unwrap_variant, metadata, var_name, eval_error_type)] =>
    #[proc_macro_error]
    evaluate::derive
}

mod attribute_dsl;

mod attribute_derives;
decl_derive! { [Attributed, attributes(check, check_type, synth, synth_type, bidir_type)] =>
    #[proc_macro_error]
    attribute_derives::derive_attributed
}

mod resolve_vars;
decl_derive! { [ResolveVars, attributes(var_index, binding, var_name, binding_name)] => resolve_vars::derive }
