use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Type, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma,
};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::{
    attributes::{
        BINDING_ATTR, DEBRUIJN_VAR_ATTR, SUBST_TYPES_ATTR, VAR_WRAPPER_ATTR,
    },
    utils::{BindingInfoExt, HasAttributes, VariantInfoExt, type_ident},
};

struct SubstDerive<'a> {
    ast: &'a Structure<'a>,
    ty: Type,
}

impl SubstDerive<'_> {
    // fn is_self_subst(&self) -> bool {
    //     // todo: Better way to compare types?
    //     self.ty == parse_quote!(Self)
    //         || self.ty == type_ident(self.ast.ast().ident.clone())
    // }

    fn ast_is_var_wrapper(&self) -> bool {
        self.ast.variants().iter().all(|variant| {
            variant.bindings().iter().any(|binding| {
                binding.has_attribute(DEBRUIJN_VAR_ATTR)
                    || binding.has_attribute(VAR_WRAPPER_ATTR)
            })
        })
    }

    fn subst_target_type(&self) -> Type {
        if self.ast_is_var_wrapper() {
            self.ty.clone()
        } else {
            type_ident(self.ast.ast().ident.clone())
        }
    }

    fn debruijn_index_variant_impl(
        &self,
        variant: &VariantInfo<'_>,
        var_field: BindingInfo<'_>,
    ) -> TokenStream {
        let greater_than_case = variant.construct_from_bindings(|binding| {
            if *binding == var_field {
                quote! { *#binding - 1 }
            } else {
                quote! { ::std::clone::Clone::clone(#binding) }
            }
        });

        let subst_ty = &self.ty;
        let target_ty = self.subst_target_type();
        let equal_case = quote! {{
            let __tmp = spez::spez! {
                for _ast_param_other;
                match<T: ::std::convert::Into<#target_ty> + Clone> &T -> ::std::option::Option<#target_ty> {
                    ::std::option::Option::Some(
                    ::std::convert::Into::into(
                    ::std::clone::Clone::clone(
                        _ast_param_other
                    )))
                }

                match<T> &T -> ::std::option::Option<#target_ty> {
                    ::std::option::Option::None
                }
            };
            match __tmp {
                ::std::option::Option::Some(__tmp) => __tmp,
                ::std::option::Option::None =>
                    return ::std::result::Result::Err(
                        ::ttt::SubstError::new::<#subst_ty, #target_ty>()
                    )
            }
        }};

        let less_than_case = variant.construct_from_bindings(|x| {
            quote! {
                ::std::clone::Clone::clone(#x)
            }
        });

        quote! {{
            ::std::result::Result::Ok(
                match std::cmp::Ord::cmp(#var_field, &_ast_param_var) {
                    std::cmp::Ordering::Less => #less_than_case.into(),
                    std::cmp::Ordering::Equal => #equal_case,
                    std::cmp::Ordering::Greater => #greater_than_case.into(),
                }
            )
        }}
    }

    fn variable_variant_impl(
        &self,
        variant: &VariantInfo<'_>,
        var_field: BindingInfo<'_>,
    ) -> TokenStream {
        let greater_than_case = variant.construct_from_bindings(|binding| {
            if *binding == var_field {
                quote! {
                    ::ttt::DeBruijnIndexed::map_indices(#var_field, |index| { index - 1 })
                }
            } else {
                quote! {
                    ::std::clone::Clone::clone(#binding)
                }
            }
        });

        let subst_ty = &self.ty;
        let target_ty = self.subst_target_type();
        let equal_case = quote! {{
            let __tmp = spez::spez! {
                for _ast_param_other;
                match<T: ::std::convert::Into<#target_ty> + Clone> &T -> ::std::option::Option<#target_ty> {
                    ::std::option::Option::Some(
                    ::std::convert::Into::into(
                    ::std::clone::Clone::clone(
                        _ast_param_other
                    )))
                }
                match<T> &T -> ::std::option::Option<#target_ty> {
                    ::std::option::Option::None
                }
            };
            match __tmp {
                ::std::option::Option::Some(__tmp) => __tmp,
                ::std::option::Option::None => {
                    return ::std::result::Result::Err(
                        ::ttt::SubstError::new::<#subst_ty, #target_ty>()
                    )
                }
            }
        }};

        let less_than_case = variant.construct_from_bindings(|binding| {
            quote! {
                ::std::clone::Clone::clone(#binding)
            }
        });

        quote! {
            {
                let index = ::ttt::DeBruijnIndexed::get_var(#var_field).unwrap();
                ::std::result::Result::Ok(
                    match ::std::cmp::Ord::cmp(&index, &_ast_param_var) {
                        ::std::cmp::Ordering::Less => #less_than_case,
                        ::std::cmp::Ordering::Equal => #equal_case,
                        ::std::cmp::Ordering::Greater => #greater_than_case,
                    }
                )
            }
        }
    }

    fn generic_variant_impl(&self, variant: &VariantInfo<'_>) -> TokenStream {
        let ctor = variant.construct_from_bindings(|binding| {
            let subst_ty = &self.ty;
            if binding.is_metadata() {
                quote! {
                    ::std::clone::Clone::clone(#binding)
                }
            } else if binding.has_attribute(BINDING_ATTR) {
                quote_spanned! { binding.ast().span() =>
                    ::ttt::Substitute::<#subst_ty>::substitute(
                        #binding,
                        &::ttt::DeBruijnIndexed::increment_indices(
                            _ast_param_other),
                        _ast_param_var + 1)?
                }
            } else {
                quote_spanned! { binding.ast().span() =>
                    ::ttt::Substitute::<#subst_ty>::substitute(
                        #binding,
                        _ast_param_other,
                        _ast_param_var)?
                }
            }
        });
        quote! {
            ::std::result::Result::Ok(#ctor)
        }
    }

    fn variant_impl(&self, variant: &VariantInfo<'_>) -> TokenStream {
        if let Some(var_field) =
            variant.find_binding_with_attribute(DEBRUIJN_VAR_ATTR)
        {
            self.debruijn_index_variant_impl(variant, var_field)
        } else if let Some(var_field) =
            variant.find_binding_with_attribute("variable")
        {
            self.variable_variant_impl(variant, var_field)
        } else {
            self.generic_variant_impl(variant)
        }
    }
}

impl ToTokens for SubstDerive<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let subst_expr_impl =
            self.ast.each_variant(|variant| self.variant_impl(variant));
        let subst_expr_impl = quote! {
            match self {
                #subst_expr_impl
            }
        };

        let subst_type = &self.ty;
        let subst_target_type = self.subst_target_type();

        // TODO: #[subst_infallible] attribute to check that all possible substitutions will be valid,
        // and set Error to ::std::convert::Infallible.
        self.ast.gen_impl(quote! {
            gen impl ::ttt::Substitute<#subst_type> for @Self {
                type Target = #subst_target_type;
                type Error = ::ttt::SubstError;

                fn substitute(&self, _ast_param_other: &#subst_type, _ast_param_var: usize) -> Result<Self::Target, Self::Error> {
                    #subst_expr_impl
                }
            }
        }).to_tokens(tokens);
    }
}

/// Extracts the list of types which can be substituted into the deriving type.
/// These should be specified as an attribute `#[SubstTypes(Type1, Type2, ...)]`
/// on the type for which substitution is being derived.
/// If SubstTypes is not specified then we use a default as if `#[SubstTypes(Self)]`.
///
fn substitutee_types(ast: &Structure) -> Vec<Type> {
    let Some(attr) = ast.find_attribute(SUBST_TYPES_ATTR) else {
        return vec![type_ident(ast.ast().ident.clone())];
    };
    let type_names = attr.meta.require_list().and_then(|list| {
        list.parse_args_with(Punctuated::<Type, Comma>::parse_terminated)
    });
    match type_names {
        Ok(type_names) => type_names.into_iter().collect(),
        Err(e) => abort!(e.span(),
            "Error while parsing `#[{}(...)]` attribute", SUBST_TYPES_ATTR;
            note = "{}", e;
            help = "The input to this attribute should be a list of type names.";
        ),
    }
}

pub fn derive(mut ast: Structure) -> TokenStream {
    ast.bind_with(|_| synstructure::BindStyle::Move);
    substitutee_types(&ast)
        .into_iter()
        .map(|ty| SubstDerive { ast: &ast, ty }.to_token_stream())
        .collect()
}
