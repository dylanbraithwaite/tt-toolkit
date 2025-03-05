//! Derive macros for the [tt-toolkit](https://crates.io/crates/tt-toolkit) crate.
//! 
//! Documentation for the macros defined here can be found in the docs for the main crate.

mod attributes;
mod utils;

use proc_macro_error2::proc_macro_error;
use synstructure::decl_derive;


mod debruijn_indexed;
decl_derive! { [DeBruijnIndexed, attributes(var_index, variable, binding, metadata)] =>
    #[proc_macro_error]
    debruijn_indexed::derive
}

mod substitute;
decl_derive! { [Substitute, attributes(var_index, subst_types, variable, binding)] =>
    #[proc_macro_error]
    substitute::derive
}

mod evaluate;
decl_derive! { [Evaluate, attributes(eval_target, context_type, binding, evaluate_with, evaluate_pattern, metadata, var_name, eval_error_type)] =>
    #[proc_macro_error]
    evaluate::derive
}

mod attribute_dsl;

#[proc_macro]
#[proc_macro_error]
pub fn attr_dsl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    attribute_dsl::attr_dsl(input.into()).into()
}

mod attribute_derives;
decl_derive! { [Attributed, attributes(check, check_type, synth, synth_type, bidir_type)] =>
    #[proc_macro_error]
    attribute_derives::derive_attributed
}
