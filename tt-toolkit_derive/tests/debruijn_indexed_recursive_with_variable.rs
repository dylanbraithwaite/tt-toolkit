use ttt::DeBruijnIndexed;

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