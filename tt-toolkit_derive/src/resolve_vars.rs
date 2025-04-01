use crate::utils::*;
use attributes::HasAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{AddBounds, BindingInfo, Structure, VariantInfo};

fn field_is_metadata(binding: &BindingInfo) -> bool {
    binding.has_attribute("metadata")
        || binding.has_attribute("var_name")
        || binding.has_attribute("binding_name")
}

fn resolve_vars_impl(ast: &mut Structure) -> TokenStream {
    ast.bind_with(|_| synstructure::BindStyle::Move);
    let match_body = ast.each_variant(resolve_vars_variant_impl);

    quote! {
        match self {
            #match_body
        }
    }
}

fn resolve_vars_variant_impl(variant: &VariantInfo) -> TokenStream {
    variant.construct_from_bindings(|field| {
        if field_is_metadata(field) {
            quote! {
                #field.clone()
            }
        } else if field.has_attribute("binding") {
            let Some(name_field) = variant
                .bindings()
                .iter()
                .find(|field| field.has_attribute("binding_name"))
            else {
                let variant = variant.ast().ident;
                panic!("In variant {variant}, expected a #[binding_name] field");
            };
            quote! {
                #field.resolve_vars(
                    &__ttt_param_ctx.append(#name_field.clone())
                )?.into()
            }
        } else if field.has_attribute("var_index") {
            let Some(name_field) = variant
                .bindings()
                .iter()
                .find(|field| field.has_attribute("var_name"))
            else {
                let variant = variant.ast().ident;
                panic!("In variant {variant}, expected a #[var_name] field");
            };
            quote! {
                ::std::iter::Iterator::position(
                    &mut ::ttt::NameContext::iter(__ttt_param_ctx),
                    |__ttt_param_name| __ttt_param_name == #name_field
                ).ok_or_else(|| ::ttt::ResolveVarsError::UnboundVariable(#name_field.to_string()))?
            }
        } else {
            quote! {
                #field.resolve_vars(__ttt_param_ctx)?.into()
                // ::ttt::ResolveVars::resolve_vars(*(#field.borrow()), __ttt_param_ctx)?.into()
            }
        }
    })
}

pub fn derive(mut ast: synstructure::Structure) -> proc_macro2::TokenStream {
    ast.add_bounds(AddBounds::Generics);
    let resolve_vars_impl = resolve_vars_impl(&mut ast);

    ast.gen_impl(quote! {
        gen impl ttt::ResolveVars for @Self {
            fn resolve_vars(&self, __ttt_param_ctx: &ttt::NameContext) -> Result<Self, ::ttt::ResolveVarsError> {
                use std::borrow::Borrow;
                Ok(#resolve_vars_impl)
            }
        }
    })
}
