/// measure the time it takes to apply a function on a set of arguments and returns the result of
/// the call
///
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let (res, time) = timeit!(add, 1, 2);
/// ```
/// will be the same as
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let (res, time) = {
///     let start = Instant::now();
///     let res = add(1, 2);
///     let time = start.elapsed();
///     (res, time)
/// };
/// ```
#[macro_export]
macro_rules! timeit {
    ($func:expr, $( $args:expr ),*) => {{
        let start = Instant::now();
        let res = $func( $( $args ),* );
        let time = start.elapsed();
        (res, time)
    }};
}

/// same as [`timeit`] but prints a name and the time at the end directly
///
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let res = timeit_and_print!("addition", add, 1, 2);
/// ```
/// will be the same as
/// ```rust
/// fn add(a: i32, b: i32) { a + b }
/// let res = {
///     print!("addition: ");
///     let start = Instant::now();
///     let res = add(1, 2);
///     let time = start.elapsed();
///     println!("{}", time.as_nanos());
///     res
/// };
/// ```
#[macro_export]
macro_rules! timeit_and_print {
    ($name: expr, $func:expr, $( $args:expr ),*) => {{
        print!("{}: ", $name);
        let (res, time) = timeit!($func, $($args),*);
        println!("{}", time.as_nanos());
        res
    }};
}
