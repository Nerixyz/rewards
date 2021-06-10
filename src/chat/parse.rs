/// Returns `Some` if there is a space in the slice. Returns `None` if there is no space.
/// The tuple halfs map right to the input.
///
/// ## Example
///
/// ```
/// assert_eq!(next_space("abc  123"), Some(("abc", "123")));
/// ```
pub fn next_space(input: &str) -> Option<(&str, &str)> {
    if let Some(idx) = input.find(' ') {
        let (left, right) = input.split_at(idx);

        Some((left.trim(), right.trim()))
    } else {
        None
    }
}

/// Always returns the left half. If there is a space, returns `Some` in the right half, else `None` is returned.
pub fn opt_next_space(input: &str) -> (&str, Option<&str>) {
    if let Some(idx) = input.find(' ') {
        let (left, right) = input.split_at(idx);

        (left.trim(), Some(right.trim()))
    } else {
        (input, None)
    }
}
