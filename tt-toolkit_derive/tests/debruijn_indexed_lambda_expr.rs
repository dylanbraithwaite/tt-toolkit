use ttt::DeBruijnIndexed;

#[derive(DeBruijnIndexed, Debug, PartialEq, Clone)]
enum LambdaExpr {
    Var(#[metadata] &'static str, #[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}

#[test]
fn shifts_under_binders() {
    let x = LambdaExpr::Var("foo", 0);
    assert_eq!(x.get_var(), Some(0));

    let x = x.increment_indices();
    assert_eq!(x.get_var(), Some(1));

    let x = LambdaExpr::Lambda(Box::new(x));
    assert_eq!(x.get_var(), None);

    let x = x.increment_indices();
    assert_eq!(x.get_var(), None);
    assert_eq!(x, LambdaExpr::Lambda(Box::new(LambdaExpr::Var("foo", 2))));
}

#[test]
fn skips_bound_variables() {
    let x = LambdaExpr::Lambda(Box::new(LambdaExpr::Var("foo", 0)));
    let y = x.clone().increment_indices();
    assert_eq!(x, y);

    let y = LambdaExpr::App(Box::new(x.clone()), Box::new(LambdaExpr::Var("bar", 0)));
    assert_eq!(y.get_var(), None);

    let y = y.increment_indices();
    assert_eq!(
        y,
        LambdaExpr::App(Box::new(x.clone()), Box::new(LambdaExpr::Var("bar", 1)))
    )
}
