use ttt::{DeBruijnIndexed, Substitute};



#[derive(Clone, DeBruijnIndexed, PartialEq, Debug, Substitute)]
enum LambdaExpr {
    Var(#[variable] Variable),
    Lambda(#[binding] Box<LambdaExpr>),
    App(Box<LambdaExpr>, Box<LambdaExpr>),
}

#[derive(Clone, Debug, PartialEq, DeBruijnIndexed, Substitute)]
#[subst_types(LambdaExpr)]
struct Variable {
    #[var_index]
    index: usize,
    #[metadata]
    name: String,
}

impl From<Variable> for LambdaExpr {
    fn from(value: Variable) -> Self {
        LambdaExpr::Var(value)
    }
}

#[test]
fn lambda_expr() {
    use LambdaExpr::*;
    let expr = App(
        Lambda(
            Var(Variable {
                index: 0,
                name: "foo".to_string(),
            })
            .into(),
        )
        .into(),
        Var(Variable {
            index: 0,
            name: "bar".to_string(),
        })
        .into(),
    );

    let var = Var(Variable {
        index: 3,
        name: "baz".to_string(),
    });

    let expr = expr.substitute(&var, 0);
    assert_eq!(
        expr,
        Ok(App(
            Lambda(
                Var(Variable {
                    index: 0,
                    name: "foo".to_string(),
                })
                .into()
            )
            .into(),
            var.into()
        ))
    )
}
