use ttt::Substitute;


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