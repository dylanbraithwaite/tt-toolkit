use ttt::{DeBruijnIndexed, Evaluate};

// #[derive(Clone, DeBruijnIndexed, Evaluate)]
// struct UnitStruct;

#[derive(Clone, DeBruijnIndexed, Evaluate)]
enum LambdaExpr {
    Var(#[var_index] usize),
    Lambda(#[binding] Box<LambdaExpr>),
    #[evaluate_pattern(
        (LambdaExpr::Lambda(body), arg) => LambdaExpr::Var(0)
    )]
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}
