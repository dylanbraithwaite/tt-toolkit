use ttt::{contextual_eq::SyntacticEq, Context, SynthAttribute, Attributed};

#[derive(Clone, PartialEq, Debug)]
enum Ty {
    Prod(Box<Ty>, Box<Ty>),
    Func(Box<Ty>, Box<Ty>),
    Unit,
}

impl SyntacticEq for Ty {}


#[derive(Clone, Attributed)]
#[synth_type(Ty)]
enum Expr {
    #[synth(Ty; _ => Ty::Unit)]
    Unit,

    #[synth(Ty; (src, body) =>
        let tgt: Ty = bind src { synth(body) };
        Ty::Func(src.clone().into(), tgt.into())
    )]
    Lam(Box<Ty>, Box<Expr>),

    #[synth(Ty; (Expr::Lam(src, body), arg) =>
        check(arg, src);
        bind src.as_ref() { synth(body) }
    )]
    App(Box<Expr>, Box<Expr>),

    #[synth(Ty; (left, right) =>
        Ty::Prod(Box::new(synth(left)), Box::new(synth(right)))
    )]
    Pair(Box<Expr>, Box<Expr>),
}

#[test]
fn check_lambda() {
    use Expr::*;
    let expr = Lam(Ty::Unit.into(), Unit.into());
    let ty = Ty::Func(Ty::Unit.into(), Ty::Unit.into());

    assert_eq!(SynthAttribute::<Ty>::synth(&expr, &Context::empty()).unwrap(), ty)
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
    assert_eq!(SynthAttribute::<Ty>::synth(&expr, &Context::empty()).unwrap(), ty)
}

#[test]
fn check_lambda_app() {
    use Expr::*;
    let expr = App(Lam(Ty::Unit.into(), Unit.into()).into(), Unit.into());
    assert_eq!(SynthAttribute::<Ty>::synth(&expr, &Context::empty()).unwrap(), Ty::Unit)
}