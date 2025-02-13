use ttt::Substitute;

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