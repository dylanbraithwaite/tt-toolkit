use ttt::{CheckAttribute, Context};
use ttt_derive::CheckAttribute;

#[derive(Clone, PartialEq)]
enum Ty {
    Prod(Box<Ty>, Box<Ty>),
    Func(Box<Ty>, Box<Ty>),
    Unit,
}


#[derive(Clone, CheckAttribute)]
#[check_type(Ty)]
enum Expr {
    #[check(Ty; () : Ty::Unit => true )]
    Unit,

    #[check(Ty; (src, body) : Ty::Func(src_, tgt) =>
        (src == src_.as_ref()) && bind src { check(body, tgt) }
    )]
    Lam(Box<Ty>, Box<Expr>),

    #[check(Ty; (Expr::Lam(src, body), arg) : ty =>
        check(arg, src) &&
        bind src.as_ref() { check(body, ty) }
    )]
    App(Box<Expr>, Box<Expr>),

    #[check(Ty; (left, right) : Ty::Prod(left_ty, right_ty) =>
        check(left, left_ty) && check(right, right_ty)
    )]
    Pair(Box<Expr>, Box<Expr>),
}

#[test]
fn check_lambda() {
    use Expr::*;
    let expr = Lam(Ty::Unit.into(), Unit.into());
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
    assert!(CheckAttribute::<Ty>::check(&expr, &ty, &Context::empty()).unwrap())
}

#[test]
fn check_lambda_app() {
    use Expr::*;
    let expr = App(Lam(Ty::Unit.into(), Unit.into()).into(), Unit.into());
    assert!(CheckAttribute::<Ty>::check(&expr, &Ty::Unit, &Context::empty()).unwrap())
}