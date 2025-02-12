use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Field, Ident, Type, TypePath, parse::Parse};
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
            std::result::Result::Ok(#self)
        }
    }

    fn _result_err<Q: Parse>(&self) -> Q {
        syn::parse_quote! {
            std::result::Result::Err(#self)
        }
    }

    fn cloned(self) -> TokenStream {
        quote! {
            std::clone::Clone::clone(&#self)
        }
    }

    fn move_cloned(self) -> TokenStream {
        quote! {
            std::clone::Clone::clone(#self)
        }
    }

    fn intoed(&self) -> TokenStream {
        syn::parse_quote! {
            std::convert::Into::into(#self)
        }
    }

    fn into_named(&self, into_type: impl ToTokens) -> TokenStream {
        syn::parse_quote! {
            std::convert::Into::<#into_type>::into(#self)
        }
    }

    fn _borrowed(&self) -> TokenStream {
        syn::parse_quote! {
            std::borrow::Borrow::borrow(&#self)
        }
    }

    fn _dereffed(&self) -> TokenStream {
        syn::parse_quote! {
            std::ops::Deref::deref(#self)
        }
    }

    fn auto_deref(&self) -> TokenStream {
        quote! {
            {
                use ::std::ops::Deref;
                (&#self).deref()
            }
        }
    }

    fn as_refed(&self) -> TokenStream {
        syn::parse_quote! {
            std::convert::AsRef::as_ref(&#self)
        }
    }

    fn into_if(&self, cond: bool) -> TokenStream {
        if cond { self.intoed() } else { quote!(#self) }
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
            let binding = bindings.iter().nth(i).unwrap();
            f(binding)
        })
    }

    fn find_binding_with_attribute(
        &self,
        attr_name: &str,
    ) -> Option<BindingInfo> {
        self.bindings()
            .iter()
            .filter(|x| x.has_attribute(attr_name))
            .next()
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

impl<'a> BindingInfoExt for BindingInfo<'a> {
    fn is_metadata(&self) -> bool {
        self.has_attribute(METADATA_ATTR)
            || self.has_attribute(VAR_NAME_ATTR)
            || self.has_attribute(BINDING_NAME_ATTR)
    }
}
