use ttt::{DeBruijnIndexed, Evaluate, Substitute};

// #[derive(Clone, DeBruijnIndexed, Evaluate)]
// struct UnitStruct;

#[derive(Clone, DeBruijnIndexed, Substitute, Evaluate, PartialEq, Debug)]
enum LambdaExpr {
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    #[evaluate_pattern {
        (Lambda(body), arg) => body.substitute(&arg, 0)?
    }]
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}

use LambdaExpr::*;

#[test]
fn evaluate_app() {
    let arg = Lambda(Var(1).into());
    let expr = App(
        Lambda(Var(0).into()).into(),
        arg.clone().into(),
    );

    let evalled = expr.evaluate_closed(false);
    assert_eq!(evalled, Ok(arg));
}
