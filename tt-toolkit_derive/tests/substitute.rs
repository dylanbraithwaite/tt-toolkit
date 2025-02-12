use ttt::{DeBruijnIndexed, Substitute};

#[derive(Substitute, Debug, PartialEq)]
struct Unit;

#[test]
fn unit() {
    let x = Unit;
    let x = x.substitute(&Unit, 0);
    assert_eq!(x, Ok(Unit))
}

#[derive(Substitute, Debug, PartialEq)]
#[subst_types(Self)]
struct UnitExplicitSelf;

#[test]
fn unit_explicit_self() {
    let x = UnitExplicitSelf;
    let x = x.substitute(&UnitExplicitSelf, 0);
    assert_eq!(x, Ok(UnitExplicitSelf));
}

#[derive(Substitute, Debug, PartialEq)]
#[subst_types(String)]
struct UnitExplicitOther;

#[test]
fn unit_explicit_other() {
    let x = UnitExplicitOther;
    let x = x.substitute(&"foo".to_string(), 0);
    assert_eq!(x, Ok(UnitExplicitOther));
}

#[derive(Substitute, Debug, PartialEq)]
#[subst_types(String, Self)]
struct UnitExplicitBoth;

#[test]
fn unit_explicit_both() {
    let x = UnitExplicitBoth;
    let x = x.substitute(&"foo".to_string(), 0);
    assert_eq!(x, Ok(UnitExplicitBoth));

    let x = x.unwrap().substitute(&UnitExplicitBoth, 1);
    assert_eq!(x, Ok(UnitExplicitBoth));
}

#[derive(Substitute, Debug, PartialEq, Clone)]
#[subst_types(VarOrString)]
struct SimpleVarWrapper(#[var_index] usize);

#[derive(Debug, PartialEq, Clone)]
enum VarOrString {
    Var(SimpleVarWrapper),
    String(String),
}

impl From<SimpleVarWrapper> for VarOrString {
    fn from(value: SimpleVarWrapper) -> Self {
        VarOrString::Var(value)
    }
}

#[test]
fn simple_var_wrapper() {
    let var = SimpleVarWrapper(0);
    let expr = VarOrString::String("foo".to_string());

    let subst1 = var.substitute(&expr, 1);
    assert_eq!(subst1, Ok(VarOrString::Var(SimpleVarWrapper(0))));

    let subst2 = var.substitute(&expr, 0);
    assert_eq!(subst2, Ok(expr))
}

#[derive(Substitute, Debug, PartialEq, Clone)]
enum RecursiveVar {
    Var(#[var_index] usize),
    Wrap(Box<RecursiveVar>),
}

impl RecursiveVar {
    fn len(&self) -> usize {
        match self {
            RecursiveVar::Var(_) => 0,
            RecursiveVar::Wrap(recursive_var) => 1 + recursive_var.len(),
        }
    }
}

#[test]
fn recursive_var() {
    use RecursiveVar::*;

    let x = Wrap(Box::new(Wrap(Box::new(Var(0)))));
    assert_eq!(x.len(), 2);

    let y = x.substitute(&x, 0).unwrap();
    assert_eq!(y.len(), 4);

    let z = y.substitute(&x, 2).unwrap();
    assert_eq!(z.len(), 4);
}

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
