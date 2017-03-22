use discord::model::ServerId;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Discord(::discord::Error);
    }

    errors {
        AssertionFailed(msg: String) {
            description("assertion failed")
            display("assertion failed: {}", msg)
        }
        ConfigError(msg: String) {
            description("configuration error")
            display("configuration error: {}", msg)
        }
        ServerConfigError(server: ServerId, msg: String) {
            description("server configuration error")
            display("server configuration error: {}", msg)
        }
        FormatError(server: ServerId, err: ::strfmt::FmtError) {
            description("format error")
            display("format error for server {}: {:?}", server, err)
        }
    }
}

macro_rules! assert {
    ($cond:expr) => (
        if !$cond {
            Err(Error::from(ErrorKind::AssertionFailed(stringify!($cond).to_string())))?;
        }
    );
    ($cond:expr, $($arg:tt)+) => (
        if !$cond {
            Err(Error::from(ErrorKind::AssertionFailed((format!($($arg)+)))))?;
        }
    );
}

macro_rules! unwrap {
    ($opt:expr) => (
        match $opt {
            Some(val) => val,
            None => Err(Error::from(ErrorKind::AssertionFailed(format!("asserted Some but got None: {}", stringify!($opt)))))?
        }
    );
    ($opt:expr, err $err:ident, $($arg:tt)+) => (
        match $opt {
            Some(val) => val,
            None => Err(Error::from(ErrorKind::$err(format!($($arg)+))))?
        }
    );
    ($opt:expr, $($arg:tt)+) => (
        match $opt {
            Some(val) => val,
            None => return Err(Error::from(ErrorKind::AssertionFailed(format!("asserted Some but got None: {}", format_args!($($arg)+)))))?
        }
    );
}

macro_rules! assert_eq {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    Err(Error::from(ErrorKind::AssertionFailed(format!(
                        "assertion failed: `(left == right)` (left: `{:?}`, right: `{:?}`)",
                         left_val, right_val)
                    )))?;
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    Err(Error::from(ErrorKind::AssertionFailed(format!(
                        "assertion failed: `(left == right)` (left: `{:?}`, right: `{:?}`): {}",
                         left_val, right_val, format_args!($($arg)+))
                     )))?;
                }
            }
        }
    });
}

macro_rules! assert_ne {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if *left_val == *right_val {
                    Err(Error::from(ErrorKind::AssertionFailed(format!(
                        "assertion failed: `(left != right)` (left: `{:?}`, right: `{:?}`)",
                         left_val, right_val)
                    )))?;
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if *left_val == *right_val {
                    Err(Error::from(ErrorKind::AssertionFailed(format!(
                        "assertion failed: `(left != right)` (left: `{:?}`, right: `{:?}`): {}",
                         left_val, right_val, format_args!($($arg)+))
                     )))?;
                }
            }
        }
    });
}
