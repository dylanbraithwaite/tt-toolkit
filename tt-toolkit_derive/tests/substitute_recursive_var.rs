use ttt::Substitute;

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
