#[macro_export]
macro_rules! true_or_err {
    ( $b:expr,$e:expr ) => {{
        if !$b {
            return Err($e);
        }
    }};
}
