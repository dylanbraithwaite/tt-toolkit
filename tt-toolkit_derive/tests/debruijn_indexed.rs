use ttt::DeBruijnIndexed;

#[derive(DeBruijnIndexed, PartialEq, Debug)]
struct JustAVariable(#[var_index] usize);

#[test]
fn just_a_variable() {
    let x = JustAVariable(0);
    let x = x.increment_indices();

    assert_eq!(x.get_var(), Some(1));

    let x = x.increment_indices_from_by(2, 1);
    assert_eq!(x.get_var(), Some(1));

    let x = x.increment_indices_from_by(1, 2);
    assert_eq!(x.get_var(), Some(3));
}

#[derive(DeBruijnIndexed, PartialEq, Debug)]
struct NestedVariable(#[variable] JustAVariable);

#[test]
fn nested_variable() {
    let x = NestedVariable(JustAVariable(0));

    let x = x.increment_indices();

    assert_eq!(x.get_var(), Some(1));
}

#[derive(DeBruijnIndexed, PartialEq, Debug)]
enum RecursiveWithVariable {
    Variable(#[var_index] usize),
    Recursive(Box<Self>),
}

#[test]
fn recursive_with_variable() {
    let x = RecursiveWithVariable::Recursive(RecursiveWithVariable::Variable(0).into());

    let x = x.increment_indices();

    assert_eq!(x.get_var(), None);
    assert_eq!(
        x,
        RecursiveWithVariable::Recursive(RecursiveWithVariable::Variable(1).into())
    )
}

#[derive(DeBruijnIndexed, Debug, PartialEq)]
struct IgnoresMetadata(#[metadata] usize);

#[test]
fn ignores_metadata() {
    let x = IgnoresMetadata(0);
    let x = x.increment_indices();

    assert_eq!(x.get_var(), None);
    assert_eq!(x, IgnoresMetadata(0));
}

#[derive(DeBruijnIndexed, Debug, PartialEq, Clone)]
enum LambdaTerm {
    Var(#[metadata] &'static str, #[var_index] usize),
    Lambda(#[binding] Box<LambdaTerm>),
    App(Box<LambdaTerm>, Box<LambdaTerm>),
}

#[test]
fn shifts_under_binders() {
    let x = LambdaTerm::Var("foo", 0);
    assert_eq!(x.get_var(), Some(0));

    let x = x.increment_indices();
    assert_eq!(x.get_var(), Some(1));

    let x = LambdaTerm::Lambda(Box::new(x));
    assert_eq!(x.get_var(), None);

    let x = x.increment_indices();
    assert_eq!(x.get_var(), None);
    assert_eq!(x, LambdaTerm::Lambda(Box::new(LambdaTerm::Var("foo", 2))));
}

#[test]
fn skips_bound_variables() {
    let x = LambdaTerm::Lambda(Box::new(LambdaTerm::Var("foo", 0)));
    let y = x.clone().increment_indices();
    assert_eq!(x, y);

    let y = LambdaTerm::App(Box::new(x.clone()), Box::new(LambdaTerm::Var("bar", 0)));
    assert_eq!(y.get_var(), None);

    let y = y.increment_indices();
    assert_eq!(
        y,
        LambdaTerm::App(Box::new(x.clone()), Box::new(LambdaTerm::Var("bar", 1)))
    )
}
