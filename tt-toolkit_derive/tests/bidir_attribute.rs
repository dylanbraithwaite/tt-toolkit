use ttt::{contextual_eq::SyntacticEq, Context, SynthAttribute, CheckAttribute};
use ttt_derive::BidirAttribute;

#[derive(Clone, PartialEq, Debug)]
enum Ty {
    Prod(Box<Ty>, Box<Ty>),
    Func(Box<Ty>, Box<Ty>),
    Unit,
}

impl SyntacticEq for Ty {}


#[derive(Clone, BidirAttribute)]
#[bidir_type(Ty)]
enum Expr {
    #[synth(Ty; _ => Ty::Unit)]
    Unit,

    #[check(Ty; body : Ty::Func(src, tgt) =>
        bind src { check(body, tgt) }
    )]
    Lam(Box<Expr>),

    #[synth(Ty; (Expr::Lam(body), arg) =>
        let Some(src): Option<Ty> = synth(arg);
        bind src { synth(body) }
    )]
    App(Box<Expr>, Box<Expr>),

    #[synth(Ty; (left, right) =>
        let Some(left_ty) = synth(left);
        let Some(right_ty) = synth(right);
        Ty::Prod(left_ty.into(), right_ty.into())
    )]
    Pair(Box<Expr>, Box<Expr>),
}

#[test]
fn check_lambda() {
    use Expr::*;
    let expr = Lam(Unit.into());
    let ty = Ty::Func(Ty::Unit.into(), Ty::Unit.into());

    assert!(CheckAttribute::<Ty>::check(&expr, &ty, &Context::empty()).unwrap())
}

#[test]
fn check_pair() {
    let expr = {
        use Expr::*;
        Pair(Unit.into(), Unit.into())
    };
    let ty = {
        use Ty::*;
        Prod(Unit.into(), Unit.into())
    };
    assert_eq!(SynthAttribute::<Option<Ty>>::synth(&expr, &Context::empty()).unwrap(), Some(ty))
}

#[test]
fn check_lambda_app() {
    use Expr::*;
    let expr = App(Lam(Unit.into()).into(), Unit.into());
    assert_eq!(SynthAttribute::<Option<Ty>>::synth(&expr, &Context::empty()).unwrap(), Some(Ty::Unit))
}