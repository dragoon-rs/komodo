#[macro_export]
macro_rules! label {
    ( $( $key:ident : $value:expr ),* $(,)? ) => {{
        let mut parts = Vec::new();
        $(
            parts.push(format!("{}: {}", stringify!($key), $value));
        )*
        &format!("{{{}}}", parts.join(", "))
    }};
}
