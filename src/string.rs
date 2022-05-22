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
