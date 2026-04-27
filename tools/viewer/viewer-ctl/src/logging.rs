//! Tagged stdout/stderr logging macros.
//!
//! No ANSI escape codes: VS Code task background pattern matchers read raw
//! stdout text and ANSI codes would prevent the regex patterns from matching.
//!
//! All three macros are crate-exported so submodules can use them as
//! `crate::info!(tag, ...)`.

#[macro_export]
macro_rules! info {
    ($tag:expr, $($arg:tt)*) => { println!("[{}] {}", $tag, format!($($arg)*)); };
}

#[macro_export]
macro_rules! warn {
    ($tag:expr, $($arg:tt)*) => { println!("[{}] WARN {}", $tag, format!($($arg)*)); };
}

#[macro_export]
macro_rules! error {
    ($tag:expr, $($arg:tt)*) => { eprintln!("[{}] ERROR {}", $tag, format!($($arg)*)); };
}
