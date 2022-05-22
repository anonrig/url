use std::borrow::Borrow;

const FILE_CODE_POINTS: [char; 4] = ['/', '\\', '?', '#'];

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

/// A string starts with a Windows drive letter if all the following are true:
/// - its length is greater than or equal to 2
/// - its first two code points are a Windows drive letter
/// - its length is 2 or its third code point is U+002F (/), U+005C (\), U+003F (?), or U+0023 (#).
pub fn starts_with_windows_drive_letter(input: &str, pointer: usize) -> bool {
    let length = input.len() - pointer;

    return length >= 2
        && is_windows_drive_letter(&input[pointer..pointer + 1])
        && (length == 2
            || FILE_CODE_POINTS.contains(input.chars().nth(pointer + 2).unwrap().borrow()));
}
