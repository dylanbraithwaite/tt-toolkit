use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Expr, Field, Ident, Type, TypePath, parse::Parse, parse_quote,
};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::attributes::{BINDING_NAME_ATTR, METADATA_ATTR, VAR_NAME_ATTR};

pub trait HasAttributes {
    fn find_attribute(&self, name: &str) -> Option<Attribute>;

    fn has_attribute(&self, name: &str) -> bool {
        self.find_attribute(name).is_some()
    }

    fn parse_attribute<T>(&self, name: &str) -> Option<T>
    where
        T: Parse,
    {
        self.find_attribute(name)
            .map(|attr| attr.parse_args().unwrap())
    }
}

impl HasAttributes for [Attribute] {
    fn find_attribute(&self, name: &str) -> Option<Attribute> {
        self.iter().find(|attr| attr.path().is_ident(name)).cloned()
    }
}

impl HasAttributes for Field {
    fn find_attribute(&self, name: &str) -> Option<Attribute> {
        self.attrs.find_attribute(name)
    }
}

impl HasAttributes for VariantInfo<'_> {
    fn find_attribute(&self, name: &str) -> Option<Attribute> {
        self.ast().attrs.find_attribute(name)
    }
}

impl HasAttributes for BindingInfo<'_> {
    fn find_attribute(&self, name: &str) -> Option<Attribute> {
        self.ast().find_attribute(name)
    }
}

impl HasAttributes for Structure<'_> {
    fn find_attribute(&self, name: &str) -> Option<Attribute> {
        self.ast().attrs.find_attribute(name)
    }
}

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

pub trait BindingInfoExt {
    fn is_metadata(&self) -> bool;
}

impl BindingInfoExt for BindingInfo<'_> {
    fn is_metadata(&self) -> bool {
        self.has_attribute(METADATA_ATTR)
            || self.has_attribute(VAR_NAME_ATTR)
            || self.has_attribute(BINDING_NAME_ATTR)
    }
}

pub fn option_none() -> Expr {
    parse_quote!(::core::option::Option::None)
}
