use attributes::HasAttributes;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens,};
use syn::{
    Expr, Ident, Type, TypePath, parse::Parse, parse_quote, spanned::Spanned
};
use synstructure::{BindingInfo, Structure, VariantInfo};

pub mod attributes;

pub fn _quote_all<I, T>(items: I) -> impl Iterator<Item = TokenStream>
where
    T: ToTokens,
    I: IntoIterator<Item = T>,
{
    items.into_iter().map(|f| quote!(#f))
}

pub trait ToTokensExt: ToTokens + Sized {
    fn result_ok<Q: Parse>(&self) -> Q {
        syn::parse_quote! {
            ::core::result::Result::Ok(#self)
        }
    }

    fn _result_err<Q: Parse>(&self) -> Q {
        syn::parse_quote! {
            ::core::result::Result::Err(#self)
        }
    }

    fn cloned(&self) -> TokenStream {
        quote! {
            ::core::clone::Clone::clone(#self)
        }
    }

    fn option_type(&self) -> TokenStream {
        quote! {
            ::core::option::Option<#self>
        }
    }

    fn intoed(&self) -> TokenStream {
        quote! {
            ::core::convert::Into::into(#self)
        }
    }
}

impl<T: ToTokens> ToTokensExt for T {}

pub fn comma_separated<T: ToTokens>(
    exprs: impl Iterator<Item = T>,
) -> TokenStream {
    quote! {
        #(#exprs, )*
    }
}

pub trait VariantInfoExt {
    fn construct_from_bindings(
        &self,
        f: impl Fn(&BindingInfo) -> TokenStream,
    ) -> TokenStream;

    fn find_binding_with_attribute(
        &self,
        attr_name: &str,
    ) -> Option<BindingInfo>;
}

pub trait StructureExt {
    fn each_construct(
        &self,
        f: impl Fn(&BindingInfo) -> TokenStream,
    ) -> TokenStream;
}

impl VariantInfoExt for VariantInfo<'_> {
    fn construct_from_bindings(
        &self,
        f: impl Fn(&BindingInfo) -> TokenStream,
    ) -> TokenStream {
        let bindings = self.bindings();
        self.construct(|_, i| {
            let binding = bindings.get(i).unwrap();
            f(binding)
        })
    }

    fn find_binding_with_attribute(
        &self,
        attr_name: &str,
    ) -> Option<BindingInfo> {
        self.bindings()
            .iter()
            .find(|x| x.has_attribute(attr_name))
            .cloned()
    }
}

impl StructureExt for Structure<'_> {
    fn each_construct(
        &self,
        f: impl Fn(&BindingInfo) -> TokenStream,
    ) -> TokenStream {
        self.each_variant(|variant| variant.construct_from_bindings(&f))
    }
}

pub fn type_ident(ident: Ident) -> Type {
    Type::Path(TypePath {
        qself: None,
        path: ident.into(),
    })
}

pub fn option_none() -> Expr {
    parse_quote!(::core::option::Option::None)
}

pub fn auto_deref(toks: impl ToTokens) -> TokenStream {
    quote_spanned! { toks.span() =>
        ::spez::spez! {
            for __ttt_param = #toks;
            match<'a, T: ::core::ops::Deref> &'a T -> &'a T::Target {
                ::core::ops::Deref::deref(__ttt_param)
            }
            match<T> T -> T { 
                __ttt_param
            }
        }
    }
}

pub fn auto_deref_for_trait(toks: impl ToTokens, trait_name: impl ToTokens) -> TokenStream {
    quote_spanned! { toks.span() =>
        {
            ::spez::spez! {
                for __ttt_param = #toks;
                match<'a, T: #trait_name> &'a T -> &'a T { 
                    __ttt_param
                }
                match<'a, T: ::core::ops::Deref> &'a T where T::Target: #trait_name -> &'a T::Target {
                    ::core::ops::Deref::deref(__ttt_param)
                }
            }
        }
    }
}

pub fn auto_deref_for_type(toks: impl ToTokens, type_name: impl ToTokens) -> TokenStream {
    quote_spanned! { toks.span() =>
        {
            ::spez::spez! {
                for __ttt_param = #toks;
                match<'a> &'a #type_name -> &'a #type_name { 
                    __ttt_param
                }
                match<'a, T: ::core::ops::Deref<Target = #type_name>> &'a T -> &'a T::Target {
                    ::core::ops::Deref::deref(__ttt_param)
                }
            }
        }
    }
}