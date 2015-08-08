#[macro_export]
macro_rules! matches {
    ($expression: expr, $($pattern:tt)+) => {
        _tt_as_expr_hack! {
            match $expression {
                $($pattern)+ => true,
                _ => false
            }
        }
    }
}


/// Work around "error: unexpected token: `an interpolated tt`", whatever that means.
#[macro_export]
macro_rules! _tt_as_expr_hack(
    ($value:expr) => ($value)
);


#[test]
fn it_works() {
    let foo = Some("-12");
    assert!(matches!(foo, Some(bar) if
        matches!(bar.as_bytes()[0], b'+' | b'-') &&
        matches!(bar.as_bytes()[1], b'0'...b'9')
    ));
}
