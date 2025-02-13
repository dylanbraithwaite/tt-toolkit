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