#[macro_export]
macro_rules! svlog {
    ( $( $arg:expr ),* $(,)?) => {
        {
            let mut cmd = Command::cargo_bin("svlog")?;
            cmd.env("SOCKLOG_LOG_DIR", format!("{}/tests/socklog/", env!("CARGO_MANIFEST_DIR")));
            cmd.arg("--no-pager");
            $( cmd.arg($arg); )*
            cmd.assert()
        }
    };
}

#[macro_export]
macro_rules! contains_all {
    ($arg:expr) => {
        predicate::str::contains($arg)
    };
    ($arg1:expr, $( $rest:expr ),* $(,)?) => {
        {
            predicate::str::contains($arg1).and(
                contains_all!($($rest),*)
            )
        }
    };
}
