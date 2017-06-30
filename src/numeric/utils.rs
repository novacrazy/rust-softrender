//! Utilities

/// Find minimum of two values
pub fn min<T>(a: T, b: T) -> T where T: PartialOrd {
    if a < b { a } else { b }
}