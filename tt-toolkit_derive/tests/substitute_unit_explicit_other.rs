use ttt::Substitute;


#[derive(Substitute, Debug, PartialEq)]
#[subst_types(String)]
struct UnitExplicitOther;

#[test]
fn unit_explicit_other() {
    let x = UnitExplicitOther;
    let x = x.substitute(&"foo".to_string(), 0);
    assert_eq!(x, Ok(UnitExplicitOther));
}