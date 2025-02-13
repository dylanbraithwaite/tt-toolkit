use ttt::Substitute;

#[derive(Substitute, Debug, PartialEq)]
#[subst_types(Self)]
struct UnitExplicitSelf;

#[test]
fn unit_explicit_self() {
    let x = UnitExplicitSelf;
    let x = x.substitute(&UnitExplicitSelf, 0);
    assert_eq!(x, Ok(UnitExplicitSelf));
}