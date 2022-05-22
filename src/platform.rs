/// A Windows drive letter is two code points, of which the first is an ASCII alpha and the second is either U+003A (:) or U+007C (|).
pub fn is_windows_drive_letter(input: &str) -> bool {
    input.len() == 2
        && input.chars().next().unwrap().is_ascii_alphanumeric()
        && (input.chars().nth(1) == Some(':') || input.chars().nth(1) == Some('|'))
}

/// A normalized Windows drive letter is a Windows drive letter of which the second code point is U+003A (:).
pub fn is_normalized_windows_drive_letter(input: &str) -> bool {
    input.chars().nth(1) == Some(':') && is_windows_drive_letter(input)
}
