#[macro_export]
macro_rules! time_dbg {
    ($($expr:expr),+) => {{
        #[cfg(debug_assertions)]
        {
            use std::time::Instant;
            let start = Instant::now();
            let result = ($($expr),+);
            let duration = start.elapsed();
            println!("[{}:{}] {} \ntook {:?}\n\n", file!(), line!(), stringify!($($expr),+), duration);
            result
        }
        #[cfg(not(debug_assertions))]
        {
            ($($expr),+)
        }
    }};
}