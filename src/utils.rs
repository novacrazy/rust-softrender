/// Clamp a value to the given range
///
/// ```
/// use combustion_common::num_utils::clamp;
///
/// assert_eq!(clamp(15u32, 0, 5), 5);
/// ```
pub fn clamp<T>(value: T, min: T, max: T) -> T where T: PartialOrd {
    if value < min { min } else if value > max { max } else { value }
}