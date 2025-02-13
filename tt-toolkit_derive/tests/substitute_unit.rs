use ttt::Substitute;

#[derive(Substitute, Debug, PartialEq)]
struct Unit;

#[test]
fn unit() {
    let x = Unit;
    let x = x.substitute(&Unit, 0);
    assert_eq!(x, Ok(Unit))
}