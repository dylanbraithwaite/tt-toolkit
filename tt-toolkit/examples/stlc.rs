use ttt::{
    Attributed, CheckAttribute, Context, DeBruijnIndexed, ListContext,
    PartialSynthAttribute, SynthAttribute,
    contextual_eq::{AutoContextualEq, SyntacticEq},
};

#[derive(Clone, PartialEq, Debug)]
enum Ty {
    Prod(Box<Ty>, Box<Ty>),
    Func(Box<Ty>, Box<Ty>),
    Unit,
}

impl<E, C: Context<E>> AutoContextualEq<E, C> for Ty {
    type Impl = SyntacticEq<Ty>;
}

#[derive(Clone, Attributed, DeBruijnIndexed)]
#[bidir_type(Ty)]
enum Expr {
    #[synth(Ty; _ => Ty::Unit)]
    Unit,

    #[synth(Ty; var => lookup(*var) )]
    Var(#[var_index] usize),

    #[check(Ty; body : Ty::Func(src, tgt) =>
        bind src { check(body, tgt) }
    )]
    Lam(Box<Expr>),

    #[synth(Ty; (Expr::Lam(body), arg) =>
        let Some(src): Option<Ty> = try_synth(arg);
        bind src { try_synth(body) }
    )]
    App(Box<Expr>, Box<Expr>),

    #[synth(Ty; (left, right) =>
        let Some(left_ty) = try_synth(left);
        let Some(right_ty) = try_synth(right);
        Ty::Prod(left_ty.into(), right_ty.into())
    )]
    Pair(Box<Expr>, Box<Expr>),
}

fn main() {
    check_lambda();
    check_lambda_app();
    check_pair();

    let ty = Ty::Func(Ty::Unit.into(), Ty::Unit.into());
    let ctx = ListContext::<Ty>::empty().append(ty.clone());
    let expr = Expr::Var(0);
    assert!(Expr::check(&expr, &ty, &ctx).unwrap());

    println!("Success");
}

fn check_lambda() {
    use Expr::*;
    let expr = Lam(Unit.into());
    let ty = Ty::Func(Ty::Unit.into(), Ty::Unit.into());

    assert!(
        CheckAttribute::<Ty>::check(&expr, &ty, &Context::empty()).unwrap()
    );
}

fn check_pair() {
    let expr = {
        use Expr::*;
        Pair(Unit.into(), Unit.into())
    };
    let ty = {
        use Ty::*;
        Prod(Unit.into(), Unit.into())
    };
    assert_eq!(
        PartialSynthAttribute::<Ty>::try_synth(&expr, &Context::empty())
            .unwrap(),
        Some(ty)
    )
}

fn check_lambda_app() {
    use Expr::*;
    let expr = App(Lam(Unit.into()).into(), Unit.into());
    assert_eq!(
        PartialSynthAttribute::<Ty>::try_synth(&expr, &Context::empty())
            .unwrap(),
        Some(Ty::Unit)
    )
}
