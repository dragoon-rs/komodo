/// `name_some_pair!(foo)` -> `("foo", Some(foo))`
#[macro_export]
macro_rules! name_some_pair {
    ($n:ident) => {
        (stringify!($n), Some($n))
    };
}

/// `name_none_pair!(foo)` -> `("foo", None)`
#[macro_export]
macro_rules! name_none_pair {
    ($n:ident) => {
        (stringify!($n), None)
    };
}
