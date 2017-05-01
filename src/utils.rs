//! Utilities

/// Clamp a value to the given range
pub fn clamp<T>(value: T, min: T, max: T) -> T where T: PartialOrd {
    if value < min { min } else if value > max { max } else { value }
}