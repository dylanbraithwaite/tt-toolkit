use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Arm, Expr, Field, Ident, Type, parse_quote};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::{attributes::*, utils::*};

fn under_binders_param() -> Ident {
    parse_quote!(__ttt_under_binders)
}

fn context_param() -> Ident {
    parse_quote!(__ttt_ctx)
}

fn field_doesnt_eval(field: &Field) -> bool {
    field.has_attribute(METADATA_ATTR)
        || field.has_attribute(DEBRUIJN_VAR_ATTR) // todo: remove
        || field.has_attribute(VAR_NAME_ATTR)
        || field.has_attribute(BINDING_NAME_ATTR)
    // || field_has_attribute(field, "variable")
}

fn evaluated(expr: impl ToTokens, ctx: impl ToTokens) -> TokenStream {
    let under_binders = under_binders_param();
    quote! {
        ::ttt::Evaluate::evaluate(#expr, #ctx, #under_binders)?
    }
}

fn normalised(expr: impl ToTokens, ctx: impl ToTokens) -> TokenStream {
    let under_binders = under_binders_param();
    quote! {
        ::ttt::Evaluate::normalise(#expr, #ctx, #under_binders)?
    }
}

fn ctx_consed(
    ctx: impl ToTokens,
    binding_type_expr: impl ToTokens,
) -> TokenStream {
    quote! {
        &::ttt::Context::append(
            #ctx,
            #binding_type_expr
        )
    }
}

fn evaluator_func_opt(variant: &VariantInfo) -> Option<Expr> {
    variant.parse_attribute(EVAL_FUNC_ATTR)
}

fn evaluator_pattern_opt(variant: &VariantInfo) -> Option<Arm> {
    variant.parse_attribute(EVAL_PATTERN_ATTR)
}

fn eval_under_binder(binding: &BindingInfo) -> TokenStream {
    let under_binders = under_binders_param();
    let ctx = ctx_consed(context_param(), option_none());

    let evalled = evaluated(binding, ctx);
    let not_evalled = binding.cloned();

    quote! {
        if #under_binders {
            #evalled
        } else {
            #not_evalled
        }
    }
}

fn normalise_under_binder(binding: &BindingInfo) -> TokenStream {
    let under_binders = under_binders_param();
    let ctx = ctx_consed(context_param(), option_none());

    let evalled = normalised(binding, ctx);
    let not_evalled = binding.cloned();

    quote! {
        if #under_binders {
            #evalled
        } else {
            #not_evalled
        }
    }
}

fn recursively_evalled_fields(
    variant: &VariantInfo,
) -> impl Iterator<Item = TokenStream> {
    variant
        .bindings()
        .iter()
        .map(|binding| recursively_eval_field(binding))
}

fn _recursively_normalised_fields(
    variant: &VariantInfo,
) -> impl Iterator<Item = TokenStream> {
    variant
        .bindings()
        .iter()
        .map(|binding| recursively_normalise_field(binding))
}

fn recursively_eval_field(binding: &BindingInfo) -> TokenStream {
    if binding.has_attribute("binding") {
        eval_under_binder(binding)
    } else if field_doesnt_eval(binding.ast()) {
        binding.cloned().to_token_stream()
    } else {
        evaluated(binding, context_param())
    }
}

fn recursively_normalise_field(binding: &BindingInfo) -> TokenStream {
    if binding.has_attribute(BINDING_ATTR) {
        normalise_under_binder(binding)
    } else if field_doesnt_eval(binding.ast()) {
        binding.cloned().to_token_stream()
    } else {
        normalised(binding, context_param())
    }
}

fn function_call<Args>(func_name: &impl ToTokens, args: Args) -> TokenStream
where
    Args: Iterator,
    Args::Item: ToTokens,
{
    let args = args.into_iter();
    quote! {
        #func_name(#(#args, )*)
    }
}

fn variant_impl_from_function(
    variant: &VariantInfo,
    evaluate_fn: Expr,
) -> TokenStream {
    let field_exprs = recursively_evalled_fields(variant);
    let custom_evalled = function_call(&evaluate_fn, field_exprs);
    let custom_evalled = quote!(&#custom_evalled);
    evaluated(custom_evalled, context_param())
}

fn variant_impl_from_pattern(
    variant: &VariantInfo,
    mut evaluator_arm: Arm,
) -> TokenStream {
    let evaluator_arm_body = evaluator_arm.body;
    let evaluator_arm_body = quote!(&#evaluator_arm_body);

    evaluator_arm.body =
        evaluated(evaluator_arm_body, context_param()).result_ok();

    let field_exprs = recursively_evalled_fields(variant);
    let field_names = variant.bindings();

    let ctor: TokenStream = variant
        .construct_from_bindings(|binding| binding.intoed())
        .result_ok();

    let default_arm: Arm = parse_quote! {
        (#(#field_names),*) => #ctor
    };

    quote! {
        match (#(#field_exprs),*) {
            #evaluator_arm,
            #default_arm,
        }
    }
}

fn variant_impl_unwrap(variant: &VariantInfo) -> TokenStream {
    comma_separated(recursively_evalled_fields(variant)).result_ok()
}

fn variant_impl_default(variant: &VariantInfo) -> TokenStream {
    variant
        .construct_from_bindings(|binding| recursively_normalise_field(binding))
        .result_ok()
}

fn evaluate_variant_impl(variant: &VariantInfo<'_>) -> TokenStream {
    if variant.has_attribute(EVAL_UNWRAP_ATTR) {
        variant_impl_unwrap(variant)
    } else if let Some(evaluator_fn) = evaluator_func_opt(variant) {
        variant_impl_from_function(variant, evaluator_fn)
    } else if let Some(evaluator_arm) = evaluator_pattern_opt(variant) {
        variant_impl_from_pattern(variant, evaluator_arm)
    } else {
        variant_impl_default(variant)
    }
}

fn default_context_type(eval_type: &Type) -> Type {
    let eval_type = eval_type.option_type();
    parse_quote! {
        ::ttt::ListContext<#eval_type>
    }
}

pub fn derive(mut ast: Structure) -> TokenStream {
    ast.bind_with(|_| synstructure::BindStyle::Move);
    let eval_type: Type = ast
        .parse_attribute(EVAL_TARGET_ATTR)
        .unwrap_or(parse_quote!(Self));

    let context_type: Type = ast
        .parse_attribute(CONTEXT_TYPE_ATTR)
        .unwrap_or_else(|| default_context_type(&eval_type));

    let eval_error_type: Type = ast
        .parse_attribute(EVAL_ERROR_ATTR)
        .unwrap_or(parse_quote!(::ttt::EvalError));

    let eval_impl = ast.each_variant(evaluate_variant_impl);
    let eval_impl = quote! {
        match self {
            #eval_impl
        }
    };

    let context_name = context_param();
    let under_binders_name = under_binders_param();

    ast.gen_impl(quote! {
        gen impl ::ttt::Evaluate for @Self {
            type Target = #eval_type;
            type Error = #eval_error_type;
            type Context = #context_type;

            fn evaluate(&self, #context_name: &Self::Context, #under_binders_name: bool) -> Result<Self::Target, Self::Error> {
                #eval_impl
            }
        }
    })
}
