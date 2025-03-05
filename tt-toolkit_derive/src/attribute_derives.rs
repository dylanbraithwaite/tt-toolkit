use derive_syn_parse::Parse;
use proc_macro_error2::abort;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use structmeta::StructMeta;
use syn::{parse_quote, Arm, Attribute, Ident, Pat, Token, Type};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::utils::attributes::HasAttributes;
use crate::utils::auto_deref;

const CHECK_TYPES_ATTR: &str = "check_type";
const SYNTH_TYPES_ATTR: &str = "synth_type";
const BIDIR_TYPES_ATTR: &str = "bidir_type";

#[derive(Parse)]
struct SynthBlock {
    attr_type: Type,
    _semicolon: Token![;],
    arm: SynthArm,
}

#[derive(Parse)]
struct CheckBlock {
    attr_type: Type,
    _semicolon: Token![;],
    arm: CheckArm,
}

struct BidirSynthBlock(SynthBlock);

#[derive(Parse)]
struct SynthArm {
    #[call(Pat::parse_single)]
    pattern: Pat,
    _arrow: Token![=>],
    body: TokenStream,
}

#[derive(Parse)]
struct CheckArm {
    #[call(Pat::parse_single)]
    expr_pat: Pat,
    _colon: Token![:],
    #[call(Pat::parse_single)]
    type_pat: Pat,
    _arrow: Token![=>],
    body: TokenStream,
}

trait DesugarsToMatchArm {
    fn desugar(&self, context_type: &Type, entry_type: &Type) -> Arm;

    fn generate_match(
        &self,
        match_on: impl ToTokens,
        context_type: &Type,
        entry_type: &Type,
    ) -> TokenStream {
        let arm = self.desugar(context_type, entry_type);
        // TODO: Erorr handling
        let err = quote! {
            panic!()
        };

        quote! {
            match #match_on {
                #arm,
                #[allow(unreachable_code)]
                _ => ::std::result::Result::Err(#err),
            }
        }
    }
}

fn instantiate_dsl(
    context_type: &Type,
    context: &Ident,
    attr_type: &Type,
    entry_type: &Type,
    body: impl ToTokens,
) -> TokenStream {
    quote! {
        {
        ::ttt_derive::attr_dsl! {
            context_type = #context_type;
            context = #context;
            attr_type = #attr_type;
            context_entry_type = #entry_type;
            #body
        }
        }
    }
}

impl DesugarsToMatchArm for SynthBlock {
    fn desugar(&self, context_type: &Type, entry_type: &Type) -> Arm {
        let pattern = &self.arm.pattern;
        let body = instantiate_dsl(context_type, &ctx_name(), &self.attr_type, entry_type, &self.arm.body,);
        parse_quote! {
            #pattern => ::core::result::Result::Ok(#body)
        }
    }
}

impl DesugarsToMatchArm for CheckBlock {
    fn desugar(&self, context_type: &Type, entry_type: &Type) -> Arm {
        let expr_pat = &self.arm.expr_pat;
        let type_pat = &self.arm.type_pat;
        let body = instantiate_dsl(context_type, &ctx_name(), &self.attr_type, entry_type, &self.arm.body);
        parse_quote! {
            (#expr_pat, #type_pat) => ::core::result::Result::Ok(#body)
        }
    }
}

impl DesugarsToMatchArm for BidirSynthBlock {
    fn desugar(&self, context_type: &Type, entry_type: &Type) -> Arm {
        let pattern = &self.0.arm.pattern;
        let attr_type = &self.0.attr_type;
        let optional_attr_type: Type = parse_quote!(::core::option::Option<#attr_type>);
        let body = instantiate_dsl(context_type, &ctx_name(), &optional_attr_type, entry_type, &self.0.arm.body);
        parse_quote! {
            #pattern => ::core::result::Result::Ok({
                let __ttt_param = #body;
                ::ttt::spez::spez! {
                    for __ttt_param;
                    match #attr_type -> #optional_attr_type { ::core::option::Option::Some(__ttt_param) }
                    match #optional_attr_type -> #optional_attr_type { __ttt_param }
                }
            })
        }
    }
}

fn ctx_name() -> Ident {
    parse_quote!(__ttt_context)
}

#[derive(StructMeta)]
struct AttrType {
    #[struct_meta(unnamed)]
    attr: Type,
    context_entry: Option<Type>,
    context: Option<Type>,
}

struct AttrSpec {
    attr_type: Type,
    context_entry: Type,
    context: Type,
}

impl From<AttrType> for AttrSpec {
    fn from(attr_type: AttrType) -> Self {
        let context_entry = attr_type
            .context_entry
            .unwrap_or_else(|| attr_type.attr.clone());
        let context = attr_type.context.unwrap_or_else(
            || parse_quote!(::ttt::ListContext<#context_entry>),
        );
        let attr_type = attr_type.attr;
        AttrSpec {
            attr_type,
            context_entry,
            context,
        }
    }
}

impl TryFrom<Attribute> for AttrSpec {
    fn try_from(value: Attribute) -> syn::Result<Self> {
        Ok(value.parse_args::<AttrType>()?.into())
    }

    type Error = syn::Error;
}

fn attr_types(
    structure: &Structure,
    attr_name: &str,
) -> Vec<AttrSpec> {
    let attrs: syn::Result<Vec<AttrSpec>> = structure
        .find_all_attributes(attr_name)
        .into_iter()
        .map(AttrSpec::try_from)
        .collect();

    match attrs {
        Err(e) => abort!(
            e.span(),
            "Error while parsing #[{}(...)] attribute", attr_name;
            note = "{}", e;
            help = "The input to this attribute should be a list of type names.";
        ),
        Ok(parsed) => parsed,
    }
}

fn opt_single_binding<'a>(
    variant: &'a VariantInfo,
) -> Option<&'a BindingInfo<'a>> {
    let bindings = variant.bindings();
    if bindings.len() == 1 {
        bindings.first()
    } else {
        None
    }
}


fn opt_check_clause(variant: &VariantInfo, attr_type: &Type) -> Option<CheckBlock> {
    variant.parse_all_attributes::<CheckBlock>("check")
        .into_iter()
        .find(|attr| attr.attr_type == *attr_type)
}

fn opt_synth_clause(variant: &VariantInfo, attr_type: &Type) -> Option<SynthBlock> {
    variant.parse_all_attributes::<SynthBlock>("synth")
        .into_iter()
        .find(|attr| attr.attr_type == *attr_type)
}

fn opt_bidir_synth_clause(variant: &VariantInfo, attr_type: &Type) -> Option<BidirSynthBlock> {
    opt_synth_clause(variant, attr_type).map(BidirSynthBlock)
}

// fn auto_evaluate(binding: &BindingInfo) -> TokenStream {
//     // ::ttt::Evaluate::evaluate(#expr, #ctx, #under_binders)?
//     quote_spanned! { binding.span() =>
//         ::ttt::spez::spez! {
//             for __ttt_param = #binding;
//             match<'a, T: ::core::ops::Deref> &'a T where T::Target: ::ttt::Evaluate -> &'a T::Target {
//                 ::ttt::Evaluate::evaluate(
//                     ::core::ops::Deref::deref(__ttt_param),

//             }
//             match<T> T -> T { 
//                 __ttt_param
//             }
//         }
//     }
// }



fn derive_check_one(input: &Structure, instance: AttrSpec) -> TokenStream {
    let attr_type = &instance.attr_type;
    let context_entry = &instance.context_entry;
    let context_type = &instance.context;

    let check_impl = input.each_variant(|variant| {
        let ctx_name = ctx_name();
        let attr_val = quote!(__ttt_check_value);
        if let Some(check) = opt_check_clause(variant, attr_type) {
            let bindings = variant
                .bindings()
                .iter()
                .map(auto_deref);
            let bindings = quote! { 
                (
                    ( #(#bindings),* ), 
                    #attr_val 
                ) 
            };
            check.generate_match(bindings, &instance.context, &instance.context_entry)
        } else if let Some(node) = opt_single_binding(variant) {
            quote! {
                #node.check(#ctx_name, __astlib_param_check_type)
            }
        } else {
            abort! {
                variant.ast().ident.span(),
                "Variant must specify a #[check(...)] attribute"
            }
        }
    });

    input.gen_impl(quote! {
        gen impl ::ttt::CheckAttribute<#attr_type> for @Self {
            type Ctx = #context_type;
            type Entry = #context_entry;
            type Error = ::ttt::DefaultError;
            type Check = bool;

            fn check(&self,
                __ttt_check_value: &#attr_type,
                __ttt_context: &#context_type,
            ) -> ::core::result::Result<Self::Check, Self::Error> {
                match self {
                    #check_impl
                }
            }
        }
    })
}

fn derive_synth_one(input: &Structure, instance: AttrSpec) -> TokenStream {
    let attr_type = &instance.attr_type;
    let context_entry = &instance.context_entry;
    let context_type = &instance.context;

    let synth_impl = input.each_variant(|variant| {
        let ctx_name = ctx_name();
        if let Some(synth) = opt_synth_clause(variant, attr_type) {
            let bindings = variant
                .bindings()
                .iter()
                .map(auto_deref);
            let bindings = quote! { 
                (
                   #(#bindings),*
                ) 
            };
            synth.generate_match(bindings, &instance.context, &instance.context_entry)
        } else if let Some(node) = opt_single_binding(variant) {
            quote! {
                #node.synth(#ctx_name)
            }
        } else {
            abort! {
                variant.ast().ident.span(),
                "Variant must specify a #[synth(...)] attribute"
            }
        }
    });

    input.gen_impl(quote! {
        gen impl ::ttt::SynthAttribute<#attr_type> for @Self {
            type Ctx = #context_type;
            type Entry = #context_entry;
            type Error = ::ttt::DefaultError;
            
            fn synth(&self,
                __ttt_context: &#context_type,
            ) -> ::core::result::Result<#attr_type, Self::Error> {
                match self {
                    #synth_impl
                }
            }
        }
    })
}

fn derive_bidir_one(input: &Structure, instance: AttrSpec) -> TokenStream {
    let attr_type = &instance.attr_type;
    let context_entry = &instance.context_entry;
    let context_type = &instance.context;

    let synth_impl = input.each_variant(|variant| {
        let ctx_name = ctx_name();
        if let Some(synth) = opt_bidir_synth_clause(variant, attr_type) {
            let bindings = variant
                .bindings()
                .iter()
                .map(auto_deref);
            let bindings = quote! { 
                (
                   #(#bindings),* 
                ) 
            };
            synth.generate_match(bindings, &instance.context, &instance.context_entry)
        } else if opt_check_clause(variant, attr_type).is_some() {
            quote! {
                ::core::result::Result::Ok(::core::option::Option::None)
            }
        } else if let Some(node) = opt_single_binding(variant) {
            quote! {
                #node.synth(#ctx_name)
            }
        } else {
            quote! {
                ::core::result::Result::Ok(::core::option::Option::None)
            }
        }
    });

    let check_impl = input.each_variant(|variant| {
        let ctx_name = ctx_name();
        let attr_val = quote!(__ttt_check_value);
        if let Some(check) = opt_check_clause(variant, attr_type) {
            let bindings = variant
                .bindings()
                .iter()
                .map(auto_deref);
            let bindings = quote! { 
                (
                    ( #(#bindings),* ), 
                    #attr_val 
                ) 
            };
            check.generate_match(bindings, &instance.context, &instance.context_entry)
        } else if let Some(synth) = opt_bidir_synth_clause(variant, attr_type) {
            let bindings = variant
                .bindings()
                .iter()
                .map(auto_deref);
            let bindings = quote! { 
                ( #(#bindings),* ) 
            };
            let synth_expr = synth.generate_match(bindings, &instance.context, &instance.context_entry);
            let synth_expr = quote! {
                match {#synth_expr}? {
                    ::core::option::Option::Some(__ttt_param) => __ttt_param,
                    ::core::option::Option::None => panic!()
                }
            };
            quote! {
                ::core::result::Result::Ok(
                    ::ttt::ContextualEq::<#context_entry, #context_type>::equiv(#ctx_name, #attr_val, &#synth_expr)?
                )
            }
        } else if let Some(node) = opt_single_binding(variant) {
            quote! {
                #node.check(#ctx_name, __astlib_param_check_type)
            }
        } else {
            abort! {
                variant.ast().ident.span(),
                "Variant must specify a #[check(...)] or #[synth(...)] attribute"
            }
        }
    });

    input.gen_impl(quote! {
        gen impl ::ttt::SynthAttribute<::core::option::Option<#attr_type>> for @Self {
            type Ctx = #context_type;
            type Entry = #context_entry;
            type Error = ::ttt::DefaultError;

            fn synth(&self, __ttt_context: &#context_type) 
                -> ::core::result::Result<::core::option::Option<#attr_type>, Self::Error> 
            {
                match self {
                    #synth_impl
                }
            }
        }

        gen impl ::ttt::CheckAttribute<#attr_type> for @Self {
            type Ctx = #context_type;
            type Entry = #context_entry;
            type Error = ::ttt::DefaultError;
            type Check = bool;
            
            fn check(&self,
                __ttt_check_value: &#attr_type,
                __ttt_context: &#context_type,
            ) -> ::core::result::Result<Self::Check, Self::Error> {
                match self {
                    #check_impl
                }
            }
        }

        gen impl ::ttt::BidirAttribute<#attr_type> for @Self {}
    })
}

pub fn derive_attributed(mut input: Structure) -> TokenStream {
    input.bind_with(|_| synstructure::BindStyle::Move);
   
    if !(input.has_attribute(CHECK_TYPES_ATTR) || input.has_attribute(SYNTH_TYPES_ATTR) || input.has_attribute(BIDIR_TYPES_ATTR)) {
        abort!(
            Span::call_site(),
            "Attributed derive requires at least one of the following attributes to be specified on the deriving type: {}, {}, {}",
            CHECK_TYPES_ATTR,
            SYNTH_TYPES_ATTR,
            BIDIR_TYPES_ATTR
        )
    }

    let checks = attr_types(&input, CHECK_TYPES_ATTR)
        .into_iter()
        .map(|ty| derive_check_one(&input, ty));

    let synths = attr_types(&input, SYNTH_TYPES_ATTR)
        .into_iter()
        .map(|ty| derive_synth_one(&input, ty));

    let bidirs = attr_types(&input, BIDIR_TYPES_ATTR)
        .into_iter()
        .map(|ty| derive_bidir_one(&input, ty));

    checks.chain(synths).chain(bidirs).collect()

}
