use proc_macro_error2::abort;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{ToTokens, quote, quote_spanned};
use syn::parse::{Parse, Parser};
use syn::{
    Block, Expr, Ident, Pat, Stmt, Token, Type, fold::Fold, parse_quote,
    parse_quote_spanned, spanned::Spanned, token,
};

use crate::utils::attributes::HasAttributes;
use crate::utils::{auto_deref_for_trait, auto_deref_for_type};

const WHILE_EXPR_MARKER_ATTR: &str = "ttt_while_is_bind";

fn context_name() -> Ident {
    parse_quote! {
        __ttt_ctx
    }
}

// HACK
// `bind` expressions should be parsed the same as while expressions.
//
// They should be acceptable anywhere in an expression.
// So to properly parse them with syn we would have to duplicate the whole
// higherarchy of Expr types from syn, with the extra variant.
//
// Instead we transform the TokenStream into a form which can be parsed as a regular Rust expression, by replacing the `bind` token with `while`,
// and add an attribute so we can find them again after parsing.
fn replace_bind_tokens(input: TokenStream) -> TokenStream {
    input
        .into_iter()
        .flat_map(|tok| match tok {
            TokenTree::Ident(ident) if ident == "bind" => {
                quote_spanned!(ident.span() => #[ttt_while_is_bind] while)
            }
            TokenTree::Group(group) => Group::new(
                group.delimiter(),
                replace_bind_tokens(group.stream()),
            )
            .to_token_stream(),
            tok => tok.to_token_stream(),
        })
        .collect()
}

struct ExpandBindExpressions {
    context_type: Type,
    entry_type: Type,
    context: Expr,
    attr_type: Type,
}

impl ExpandBindExpressions {
    fn expand_body(&mut self, body: Vec<Stmt>) -> TokenStream {
        let span = quote!(#(#body)*).span();

        let body = Block {
            brace_token: token::Brace(span),
            stmts: body,
        };
        let body = self.fold_block(body);

        let input_ctx = &self.context;
        let ctx_name = context_name();

        quote! {
            {
                let #ctx_name = #input_ctx;
                #body
            }
        }
    }

    fn expand_synth_expr(&self, synth_call: syn::ExprCall) -> Expr {
        let ctx_name = context_name();
        let span = synth_call.span();
        let attr_type = &self.attr_type;
        let synth_trait = &quote!(::ttt::SynthAttribute<#attr_type>);
        match synth_call.args.len() {
            1 => {
                // By default use type of current instantiation
                // I'd like to use inference here, but the current (old) trait solver consistently chokes at this
                let arg = auto_deref_for_trait(&synth_call.args, synth_trait);
                parse_quote_spanned! { span =>
                    ::ttt::SynthAttribute::<#attr_type>::synth(#arg, #ctx_name)?
                }
            }
            // Allow specifying attribute type as parameter in synth call
            2 => {
                let attr_ty = synth_call.args.first().unwrap();
                let expr = auto_deref_for_trait(synth_call.args.last().unwrap(), synth_trait);
                parse_quote_spanned! { span =>
                    ::ttt::SynthAttribute::<#attr_ty>::synth(#expr, #ctx_name)?
                }
            }
            _ => abort!(
                span,
                "`synth` call should have no more than 2 parameters"
            ),
        }
    }

    fn expand_check_expr(&self, check_call: syn::ExprCall) -> Expr {
        let ctx_name = context_name();
        let attr_ty = &self.attr_type;
        let span = check_call.span();
        let check_trait = quote!(::ttt::CheckAttribute<#attr_ty>);
        match check_call.args.len() {
            2 => {
                let mut args = check_call.args.into_iter();
                let arg1 = auto_deref_for_trait(args.next().unwrap(), check_trait);
                let arg2 = auto_deref_for_type(args.next().unwrap(), attr_ty);
                // TODO: I'd like to not have to specify attr_ty here, so that
                // check(..) calls can check other attribute types too, but
                // the compiler fails to infer this, I think due to
                // https://github.com/rust-lang/rust/issues/136856
                parse_quote_spanned! { span =>
                    ::ttt::CheckAttribute::<#attr_ty>::check(#arg1, #arg2, #ctx_name)?
                }
            }
            3 => {
                let mut args = check_call.args.into_iter();
                let attr_ty = &args.next().unwrap();
                let arg1 = auto_deref_for_trait(args.next().unwrap(), check_trait);
                let arg2 = auto_deref_for_type(args.next().unwrap(), attr_ty);
                // TODO: I'd like to not have to specify attr_ty here, so that
                // check(..) calls can check other attribute types too, but
                // the compiler fails to infer this, I think due to
                // https://github.com/rust-lang/rust/issues/136856
                parse_quote_spanned! { span =>
                    ::ttt::CheckAttribute::<#attr_ty>::check(#arg1, #arg2, #ctx_name)?
                }
            }

            _ => abort!(span, "`check` call should have exactly 2 parameters"),
        }
    }

    fn expand_lookup_expr(&self, lookup_call: syn::ExprCall) -> Expr {
        let ctx_name = &self.context;
        let ctx_ty = &self.context_type;
        // TODO: Handle case where entry type is not attr_ty.
        let entry_ty = &self.entry_type;
        let span = lookup_call.span();
        match lookup_call.args.len() {
            1 => {
                let mut args = lookup_call.args.into_iter();
                let arg = args.next().unwrap();

                // TODO: Error handling
                parse_quote_spanned! { span => {
                    let arg = #arg;
                    let entry = <#ctx_ty as ::ttt::Context<#entry_ty>>::get(#ctx_name, arg);
                    ::ttt::spez::spez! {
                        for __ttt_context = (entry, arg);
                        match<Entry: DeBruijnIndexed> (Option<Entry>, usize) -> Option<Entry> {
                            __ttt_context.0.map(|entry| {
                                ::ttt::DeBruijnIndexed::increment_indices_by(&entry, __ttt_context.1)
                            })
                        }
                        match<Entry> (Option<Entry>, usize) -> Option<Entry> {
                            __ttt_context.0
                        }
                    }
                }}
            }
            _ => abort!(span, "`lookup` call should have exactly 1 parameter"),
        }
    }

    fn expand_bind_expr(&self, bind_expr: syn::ExprWhile) -> syn::Expr {
        let span = bind_expr.span();

        let mut attrs = bind_expr.attrs;
        let attr_pos =
            attrs.attribute_position(WHILE_EXPR_MARKER_ATTR).unwrap();
        attrs.remove(attr_pos);

        let label = &bind_expr.label;
        let bindee = &bind_expr.cond;
        let body = &bind_expr.body;
        let context_ty = &self.context_type;
        // let attr_ty = self
        let context = context_name();
        parse_quote_spanned! { span =>
            #(#attrs)*
            #label
            {
                let #context =
                    &<#context_ty as ::ttt::Context<_>>::append(&#context, ::core::clone::Clone::clone(&#bindee));
                #body
            }
        }
    }
}

macro_rules! expand_calls {
    ($match_on: expr; $($ident: ident => $func: expr),*, _ => $default: expr $(,)?) => {
        match $match_on {
            $(
                Expr::Path(path) if path.path.is_ident(stringify!($ident)) => {
                    $func
                },
            )*
            _ => $default,
        }
    };
}

impl syn::fold::Fold for ExpandBindExpressions {
    fn fold_expr(&mut self, expr: syn::Expr) -> syn::Expr {
        let out = match expr {
            Expr::While(while_expr)
                if while_expr.attrs.has_attribute(WHILE_EXPR_MARKER_ATTR) =>
            {
                self.expand_bind_expr(while_expr)
            }
            Expr::Call(call_expr) => expand_calls! { call_expr.func.as_ref();
                synth => self.expand_synth_expr(call_expr),
                check => self.expand_check_expr(call_expr),
                lookup => self.expand_lookup_expr(call_expr),
                _ => Expr::Call(call_expr),
            },
            expr => expr,
        };
        syn::fold::fold_expr(self, out)
    }

    // Add else clauses to all refutable let bindings which do not already have them.
    fn fold_local(&mut self, mut local: syn::Local) -> syn::Local {
        if irrefutable_pat(&local.pat) {
            return syn::fold::fold_local(self, local);
        }

        if let Some(init) = local.init.as_mut() {
            if init.diverge.is_none() {
                let span = init.eq_token.span();
                let else_tok = Token![else](span);
                init.diverge = Some((
                    else_tok,
                    // TODO: Error handling
                    parse_quote_spanned!(span => { panic!() }),
                ));
                // In complex cases we cannot decide whether a pattern is refutable
                // without type information.
                // So add this attribute to silence compiler warnings in case we add
                // an else clause based on a false positive.
                local.attrs.push(parse_quote!(
                    #[allow(irrefutable_let_patterns)]
                ));
            }
        }

        syn::fold::fold_local(self, local)
    }
}

mod kw {
    syn::custom_keyword!(context);
    syn::custom_keyword!(context_type);
    syn::custom_keyword!(attr_type);
    syn::custom_keyword!(context_entry_type);
}

struct KeyVal<K, V> {
    _key_token: K,
    _eq_token: Token![=],
    val: V,
    _comma_token: Token![;],
}

impl<K: Parse, V: Parse> Parse for KeyVal<K, V> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(KeyVal {
            _key_token: input.parse()?,
            _eq_token: input.parse()?,
            val: input.parse()?,
            _comma_token: input.parse()?,
        })
    }
}

struct DslInstantiation {
    context_type: KeyVal<kw::context_type, Type>,
    context: KeyVal<kw::context, Expr>,
    attr_type: KeyVal<kw::attr_type, Type>,
    entry_type: KeyVal<kw::context_entry_type, Type>,
    body: TokenStream,
}

impl Parse for DslInstantiation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(DslInstantiation {
            context_type: input.parse()?,
            context: input.parse()?,
            attr_type: input.parse()?,
            entry_type: input.parse()?,
            body: input.parse()?,
        })
    }
}

fn irrefutable_pat(pat: &Pat) -> bool {
    match pat {
        Pat::Ident(syn::PatIdent { subpat: None, .. }) | Pat::Wild(_) => true,
        Pat::Paren(syn::PatParen { pat, .. })
        | Pat::Ident(syn::PatIdent {
            subpat: Some((_, pat)),
            ..
        })
        | Pat::Reference(syn::PatReference { pat, .. })
        | Pat::Type(syn::PatType { pat, .. }) => irrefutable_pat(pat),
        Pat::Tuple(pat_tuple) => pat_tuple.elems.iter().all(irrefutable_pat),
        Pat::Macro(_)
        | Pat::Const(_)
        | Pat::Lit(_)
        | Pat::Or(_)
        | Pat::Path(_)
        | Pat::Range(_)
        | Pat::Rest(_)
        | Pat::Slice(_)
        | Pat::Struct(_)
        | Pat::TupleStruct(_)
        | Pat::Verbatim(_)
        | _ => false,
    }
}

macro_rules! unwrap_parsed {
    ($ts:expr) => {
        match $ts {
            Ok(parsed) => parsed,
            Err(err) => return syn::Error::into_compile_error(err),
        }
    };
}

pub fn attr_dsl(input: TokenStream) -> TokenStream {
    let instantiation: DslInstantiation =
        unwrap_parsed!(DslInstantiation::parse.parse2(input));
    let body = unwrap_parsed!(
        Block::parse_within.parse2(replace_bind_tokens(instantiation.body))
    );

    ExpandBindExpressions {
        context_type: instantiation.context_type.val,
        context: instantiation.context.val,
        entry_type: instantiation.entry_type.val,
        attr_type: instantiation.attr_type.val,
    }
    .expand_body(body)
}
