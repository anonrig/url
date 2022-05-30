pub fn is_ascii_alphanumeric(input: Option<u8>) -> bool {
    if let Some(value) = input {
        return (value as char).is_ascii_alphanumeric();
    }

    false
}

pub fn is_ascii_digit(input: Option<u8>) -> bool {
    if let Some(value) = input {
        return (value as char).is_ascii_digit();
    }

    false
}

pub fn is_ascii_hexdigit(input: Option<u8>) -> bool {
    if let Some(value) = input {
        return (value as char).is_ascii_hexdigit();
    }

    false
}

/// A double-dot path segment must be ".." or an ASCII case-insensitive match for ".%2e", "%2e.", or "%2e%2e".
pub fn is_double_dot_path_segment(input: &str) -> bool {
    input == ".." || input == "%2e." || input == ".%2e" || input == "%2e%2e"
}

/// A single-dot path segment must be "." or an ASCII case-insensitive match for "%2e".
pub fn is_single_dot_path_segment(input: &str) -> bool {
    input == "." || input.to_lowercase() == "%2e"
}
