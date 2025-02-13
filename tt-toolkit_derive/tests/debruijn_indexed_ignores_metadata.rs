use ttt::DeBruijnIndexed;

#[derive(DeBruijnIndexed, Debug, PartialEq)]
struct IgnoresMetadata(#[metadata] usize);

#[test]
fn ignores_metadata() {
    let x = IgnoresMetadata(0);
    let x = x.increment_indices();

    assert_eq!(x.get_var(), None);
    assert_eq!(x, IgnoresMetadata(0));
}