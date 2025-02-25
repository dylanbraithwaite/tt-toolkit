use crate::utils::*;
use proc_macro_error2::abort;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::{Ident, parse_quote, spanned::Spanned};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::attributes::*;
use crate::utils::attributes::HasAttributes;

fn ensure_can_be_debruijn_var(binding: &BindingInfo) {
    let attr = binding.find_attribute(DEBRUIJN_VAR_ATTR).unwrap();
    let binding_ty = &binding.ast().ty;
    if *binding_ty != parse_quote!(usize) {
        abort!(
            attr.span(),
            "Type `{}` cannot be used as a variable index",
            binding_ty.to_token_stream();
            note = "The #[{}] annotation should only be used with fields of type usize.", DEBRUIJN_VAR_ATTR;

            help = "Maybe you meant to use the #[{}] annotation instead", VAR_WRAPPER_ATTR;
        );
    }
}

fn map_indices_impl(mut ast: Structure) -> TokenStream {
    ast.bind_with(|_| synstructure::BindStyle::Move);

    let func_type_name: Ident = parse_quote!(__TTT_F);
    let func_var_name: Ident = parse_quote!(__ttt_map_fn);
    let index_var_name: Ident = parse_quote!(__ttt_start);

    let match_body = ast.each_construct(|binding| {
        let binding_ty = &binding.ast().ty;
        let recursive_call = |index_value| {
            quote_spanned! { binding.span() =>
                <#binding_ty as ::ttt::DeBruijnIndexed>::map_indices_from(
                        #binding,
                        #index_value,
                        <#func_type_name as ::std::clone::Clone>::clone(&#func_var_name))
            }
        };

        if binding.is_metadata() {
            // This field is metadata (such as the string representation of a variable)
            // so we leave it untouched.
            quote! {
                ::std::clone::Clone::clone(#binding)
            }
        } else if binding.has_attribute(DEBRUIJN_VAR_ATTR) {
            ensure_can_be_debruijn_var(binding);
            // This field is a raw debruijn variable, so we directly modify it with the map function
            quote_spanned! { binding.span() =>
                if *#binding >= #index_var_name {
                    #func_var_name(*#binding)
                } else {
                    *#binding
                }
            }
        } else if binding.has_attribute(BINDING_ATTR) {
            // This field represents a node under a binder, so we bump the start index to account for the new variable,
            // and make a recursive call.
            recursive_call(quote!(#index_var_name + 1))
        } else {
            // This field is a regular node, so just proceed recursively
            recursive_call(index_var_name.to_token_stream())
        }
    });

    quote! {
        fn map_indices_from<#func_type_name>(&self, #index_var_name: usize, #func_var_name: #func_type_name) -> Self
        where
            #func_type_name: Fn(usize) -> usize + Clone
        {
            match self {
                #match_body
            }
        }
    }
}

fn get_var_impl(mut ast: synstructure::Structure) -> TokenStream {
    ast.bind_with(|_| synstructure::BindStyle::Move);
    let match_body = ast.each_variant(get_var_variant_impl);

    quote! {
        match self {
            #match_body
        }
    }
}

fn get_var_variant_impl(variant: &VariantInfo) -> TokenStream {
    if let Some(field) = variant.find_binding_with_attribute(VAR_WRAPPER_ATTR) {
        let field_ty = &field.ast().ty;
        quote! {
            <#field_ty as ::ttt::DeBruijnIndexed>::get_var(#field)
        }
    } else if let Some(field) =
        variant.find_binding_with_attribute(DEBRUIJN_VAR_ATTR)
    {
        quote! {
            ::std::option::Option::Some(*#field)
        }
    } else {
        quote! {
            ::std::option::Option::None
        }
    }
}

pub fn derive(ast: synstructure::Structure) -> proc_macro2::TokenStream {
    let dbn_impl = map_indices_impl(ast.clone());
    let get_var_impl = get_var_impl(ast.clone());

    ast.gen_impl(quote! {
        gen impl ::ttt::DeBruijnIndexed for @Self {
            #dbn_impl

            fn get_var(&self) -> Option<usize> {
                #get_var_impl
            }
        }
    })
}
