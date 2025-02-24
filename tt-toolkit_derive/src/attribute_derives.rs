use derive_syn_parse::Parse;
use proc_macro_error2::abort;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use structmeta::StructMeta;
use syn::{Arm, Attribute, Ident, Pat, Token, Type, parse_quote};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::utils::HasAttributes;

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
    fn desugar(&self, context_type: &Type) -> Arm;

    fn generate_match(
        &self,
        match_on: impl ToTokens,
        context_type: &Type,
    ) -> TokenStream {
        let arm = self.desugar(context_type);
        // TODO: Erorr handling
        let err = quote! {
            panic!()
        };

        quote! {
            match (#match_on) {
                #arm,
                _ => ::std::result::Result::Err(#err),
            }
        }
    }
}

fn instantiate_dsl(
    context_type: &Type,
    context: &Ident,
    body: impl ToTokens,
) -> TokenStream {
    quote! {
        {
        ::ttt_derive::attr_dsl! {
            context_type = #context_type;
            context = #context;
            #body
        }
        }
    }
}

impl DesugarsToMatchArm for SynthArm {
    fn desugar(&self, context_type: &Type) -> Arm {
        let pattern = &self.pattern;
        let body = instantiate_dsl(context_type, &ctx_name(), &self.body);
        parse_quote! {
            #pattern => #body
        }
    }
}

impl DesugarsToMatchArm for CheckArm {
    fn desugar(&self, context_type: &Type) -> Arm {
        let expr_pat = &self.expr_pat;
        let type_pat = &self.type_pat;
        let body = instantiate_dsl(context_type, &ctx_name(), &self.body);
        parse_quote! {
            (#expr_pat, #type_pat) => #body
        }
    }
}

fn ctx_name() -> Ident {
    parse_quote! {
        __ttt_context
    }
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
    derive_name: &str,
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
        Ok(parsed) if parsed.is_empty() => abort!(
            Span::call_site(),
            "{} derive requires the {} attribute to be specified on the deriving type",
            derive_name,
            attr_name
        ),
        Ok(parsed) => parsed,
    }
}

fn opt_single_binding<'a>(
    variant: &'a VariantInfo,
) -> Option<&'a BindingInfo<'a>> {
    let bindings = variant.bindings();
    if bindings.len() == 1 {
        bindings.get(0)
    } else {
        None
    }
}

fn derive_check_one(input: &mut Structure, instance: AttrSpec) -> TokenStream {
    let attr_type = &instance.attr_type;
    let context_entry = &instance.context_entry;
    let context_type = &instance.context;

    let check_impl = input.each_variant(|variant| {
        let ctx_name = ctx_name();
        if let Some(check) = variant.parse_attribute::<CheckBlock>("check") {
            let bindings = variant.bindings();
            let bindings = quote!(((#(#bindings),*), __ttt_check_value));
            check.arm.generate_match(bindings, &instance.context)
        } else if let Some(node) = opt_single_binding(&variant) {
            quote! {
                #node.check(#ctx_name, __astlib_param_check_type)
            }
        } else {
            abort! {
                variant.ast().ident.span(),
                "Variant must specify a #[check_attr(...)] attribute"
            }
        }
    });

    input.gen_impl(quote! {
        gen impl ::ttt::CheckAttribute<#attr_type> for @Self {
            type Context = #context_type;
            type Entry = #context_entry;
            type Error = ();
            type Check = bool;

            fn check(&self,
                __ttt_ctx: &#context_type,
                __ttt_check_value: Self::TypeExpr
            ) -> ::astlib::TypeCheckResult<Self> {
                match self {
                    #check_impl
                }
            }
        }
    })
}

fn derive_synth_one(input: &mut Structure, instance: AttrSpec) -> TokenStream {
    todo!()
}

fn derive_bidir_one(input: &mut Structure, instance: AttrSpec) -> TokenStream {
    todo!()
}

pub fn derive_check(mut input: Structure) -> TokenStream {
    attr_types(&input, CHECK_TYPES_ATTR, "CheckAttribute")
        .into_iter()
        .map(|ty| derive_check_one(&mut input, ty))
        .collect()
}

pub fn derive_synth(mut input: Structure) -> TokenStream {
    attr_types(&input, SYNTH_TYPES_ATTR, "SynthAttribute")
        .into_iter()
        .map(|ty| derive_synth_one(&mut input, ty))
        .collect()
}

pub fn derive_bidir(mut input: Structure) -> TokenStream {
    attr_types(&input, BIDIR_TYPES_ATTR, "BidirAttribute")
        .into_iter()
        .map(|ty| derive_bidir_one(&mut input, ty))
        .collect()
}
