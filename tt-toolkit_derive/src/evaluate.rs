use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Arm, Expr, Field, Ident, Type, parse_quote};
use synstructure::{AddBounds, BindingInfo, Structure, VariantInfo};

use crate::utils::attributes::HasAttributes;
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

fn evaluator_func_opt(variant: &VariantInfo) -> Option<Expr> {
    variant.parse_attribute(EVAL_FUNC_ATTR)
}

fn evaluator_pattern_opt(variant: &VariantInfo) -> Option<Arm> {
    variant.parse_attribute(EVAL_PATTERN_ATTR)
}

fn function_call<Args>(func_name: &impl ToTokens, args: Args) -> TokenStream
where
    Args: Iterator,
    Args::Item: ToTokens,
{
    let args = args.into_iter();
    quote! {{
        (<#func_name>)(#(#args, )*)
    }}
}

fn default_context_type(eval_type: &Type) -> Type {
    let eval_type = eval_type.option_type();
    parse_quote! {
        ::ttt::ListContext<#eval_type>
    }
}

struct EvaluateDerive<'a> {
    ast: Structure<'a>,
    eval_type: Type,
    context_type: Type,
    error_type: Type,
}

impl<'a> EvaluateDerive<'a> {
    fn ctx_consed(
        &self,
        ctx: impl ToTokens,
        binding_type_expr: impl ToTokens,
    ) -> TokenStream {
        let context_trait = self.context_trait();
        let context_type = &self.context_type;
        quote! {
            &<#context_type as #context_trait>::append(
                #ctx,
                #binding_type_expr
            )
        }
    }

    fn context_trait(&self) -> TokenStream {
        let eval_type = &self.eval_type;
        quote! {
            ::ttt::Context<Option<#eval_type>>
        }
    }

    fn recursively_evalled_fields(
        &self,
        variant: &VariantInfo,
    ) -> impl Iterator<Item = TokenStream> {
        variant
            .bindings()
            .iter()
            .map(|b| self.recursively_eval_field(b))
    }

    fn recursively_eval_field(&self, binding: &BindingInfo) -> TokenStream {
        if binding.has_attribute("binding") {
            self.eval_under_binder(binding)
        } else if field_doesnt_eval(binding.ast()) {
            binding.cloned().to_token_stream()
        } else {
            evaluated(binding, context_param())
        }
    }

    fn recursively_normalise_field(
        &self,
        binding: &BindingInfo,
    ) -> TokenStream {
        if binding.has_attribute(BINDING_ATTR) {
            self.normalise_under_binder(binding)
        } else if field_doesnt_eval(binding.ast()) {
            binding.cloned().to_token_stream()
        } else {
            normalised(binding, context_param())
        }
    }

    fn eval_under_binder(&self, binding: &BindingInfo) -> TokenStream {
        let under_binders = under_binders_param();
        let ctx = self.ctx_consed(context_param(), option_none());

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

    fn normalise_under_binder(&self, binding: &BindingInfo) -> TokenStream {
        let under_binders = under_binders_param();
        let ctx = self.ctx_consed(context_param(), option_none());

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

    fn new(mut ast: Structure<'a>) -> Self {
        ast.bind_with(|_| synstructure::BindStyle::Move);
        ast.add_bounds(AddBounds::Generics);
        let eval_type: Type = ast
            .parse_attribute_with_default(EVAL_TARGET_ATTR, || {
                parse_quote!(Self)
            });

        let context_type: Type = ast
            .parse_attribute_with_default(CONTEXT_TYPE_ATTR, || {
                default_context_type(&eval_type)
            });

        let error_type: Type = ast
            .parse_attribute_with_default(EVAL_ERROR_ATTR, || {
                parse_quote!(::ttt::EvalError)
            });

        EvaluateDerive {
            ast,
            eval_type,
            context_type,
            error_type,
        }
    }

    fn generate_evaluate_function(&self) -> TokenStream {
        let eval_impl =
            self.ast.each_variant(|var| self.evaluate_variant_impl(var));
        let context_name = context_param();
        let under_binders_name = under_binders_param();

        quote! {
            fn evaluate(&self, #context_name: &Self::Context, #under_binders_name: bool) -> Result<Self::Target, Self::Error> {
                match self {
                    #eval_impl
                }
            }
        }
    }

    fn generate_impl(&self) -> TokenStream {
        let evaluate_function = self.generate_evaluate_function();
        let eval_type = &self.eval_type;
        let eval_error_type = &self.error_type;
        let context_type = &self.context_type;

        self.ast.gen_impl(quote! {
            gen impl ::ttt::Evaluate for @Self {
                type Target = #eval_type;
                type Error = #eval_error_type;
                type Context = #context_type;

                #evaluate_function
            }
        })
    }

    fn variant_impl_from_function(
        &self,
        variant: &VariantInfo,
        evaluate_fn: Expr,
    ) -> TokenStream {
        let field_exprs = self.recursively_evalled_fields(variant);
        let field_exprs = std::iter::once(context_param().to_token_stream())
            .chain(field_exprs);

        let custom_evalled = function_call(&evaluate_fn, field_exprs);
        let custom_evalled = quote!(&(#custom_evalled?));
        evaluated(custom_evalled, context_param()).result_ok()
    }

    fn variant_impl_from_pattern(
        &self,
        variant: &VariantInfo,
        mut evaluator_arm: Arm,
    ) -> TokenStream {
        let evaluator_arm_body = evaluator_arm.body;
        let evaluator_arm_body = quote!(&#evaluator_arm_body);

        evaluator_arm.body =
            evaluated(evaluator_arm_body, context_param()).result_ok();

        let field_exprs = self.recursively_evalled_fields(variant);
        let field_names = variant.bindings();

        let ctor: TokenStream = variant
            .construct_from_bindings(|binding| binding.intoed())
            .intoed_explicit(&self.eval_type)
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

    fn variant_impl_unwrap(&self, variant: &VariantInfo) -> TokenStream {
        comma_separated(self.recursively_evalled_fields(variant)).result_ok()
    }

    fn variant_impl_default(&self, variant: &VariantInfo) -> TokenStream {
        variant
            .construct_from_bindings(|b| self.recursively_normalise_field(b))
            .intoed_explicit(&self.eval_type)
            .result_ok()
    }

    fn evaluate_variant_impl(&self, variant: &VariantInfo<'_>) -> TokenStream {
        if variant.has_attribute(EVAL_UNWRAP_ATTR) {
            self.variant_impl_unwrap(variant)
        } else if let Some(evaluator_fn) = evaluator_func_opt(variant) {
            self.variant_impl_from_function(variant, evaluator_fn)
        } else if let Some(evaluator_arm) = evaluator_pattern_opt(variant) {
            self.variant_impl_from_pattern(variant, evaluator_arm)
        } else {
            self.variant_impl_default(variant)
        }
    }
}

pub fn derive(ast: Structure) -> TokenStream {
    EvaluateDerive::new(ast).generate_impl()
}
