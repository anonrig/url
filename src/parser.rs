use crate::serializers::{serialize_ipv4, serialize_ipv6};
use idna::domain_to_ascii;
use percent_encoding::{utf8_percent_encode, CONTROLS};

/// https://url.spec.whatwg.org/#ipv4-number-parser
fn parse_ipv4_number(buffer: String) -> Option<u32> {
    let mut buffer = buffer;

    // If input is the empty string, then return failure.
    if buffer.is_empty() {
        return None;
    }

    // Let R be 10.
    let mut r = 10;

    // If input contains at least two code points and the first two code points are either "0X" or "0x", then:
    if buffer.len() >= 2 && (buffer.starts_with("0X") || buffer.starts_with("0x")) {
        // Remove the first two code points from input.
        buffer = buffer[2..].to_string();

        // Set R to 16.
        r = 16;
    }
    // Otherwise, if input contains at least two code points and the first code point is U+0030 (0), then:
    else if buffer.len() > 2 && buffer.starts_with("0") {
        // Remove the first code point from input.
        buffer = buffer[1..].to_string();

        // Set R to 8.
        r = 8;
    }

    // If input is the empty string, then return (0, true).
    if buffer.len() == 0 {
        return Some(0);
    }

    // If input contains a code point that is not a radix-R digit, then return failure.
    let radix = u32::from_str_radix(buffer.as_str(), r);

    // Let output be the mathematical integer value that is represented by input in radix-R notation, using ASCII hex digits for digits with values 0 through 15.
    if let Ok(value) = radix {
        return Some(value);
    }

    None
}

pub fn parse_ipv4(buffer: String) -> Option<u64> {
    // Let parts be the result of strictly splitting input on U+002E (.).
    let mut parts: Vec<&str> = buffer.split('.').collect();

    // If the last item in parts is the empty string, then:
    if parts.last() == Some(&"") {
        // If parts’s size is greater than 1, then remove the last item from parts.
        if parts.len() > 1 {
            parts.pop();
        }
    }

    // If parts’s size is greater than 4, validation error, return failure.
    if parts.len() > 4 {
        return None;
    }

    // Let numbers be an empty list.
    let mut numbers: Vec<u64> = vec![];

    // For each part of parts:
    for part in &parts {
        // Let result be the result of parsing part.
        let result = parse_ipv4_number(part.to_string());

        if let Some(number) = result {
            // If any item in numbers is greater than 255, validation error.
            if number > 255 && parts.last() != Some(&part) {
                return None;
            }

            // Append result[0] to numbers.
            numbers.push(number as u64)
        } else {
            // If result is failure, validation error, return failure.
            return None;
        }
    }

    // If the last item in numbers is greater than or equal to 256(5 − numbers’s size), validation error, return failure.
    if numbers.last() > Some(&256_u64.pow(5 - numbers.len() as u32)) {
        return None;
    }

    // Let ipv4 be the last item in numbers.
    // Remove the last item from numbers.
    let mut ipv4: u64 = numbers.pop().unwrap() as u64;

    // For each n of numbers:
    for (c, n) in numbers.iter().enumerate() {
        // Increment ipv4 by n × 256(3 − counter).
        ipv4 += (n.clone() as u64) * 256_u64.pow(3 - c as u32);
    }

    Some(ipv4)
}

pub fn parse_ipv6(buffer: String) -> Option<String> {
    // Let address be a new IPv6 address whose IPv6 pieces are all 0.
    let mut address: Vec<u32> = vec![0; 8];

    // Let pieceIndex be 0.
    let mut piece_index: usize = 0;

    // Let compress be null/0.
    let mut compress: Option<usize> = Some(0);

    // Let pointer be a pointer for input.
    let mut pointer = 0;

    // If c is U+003A (:), then:
    if buffer.chars().nth(pointer) == Some(':') {
        // If remaining does not start with U+003A (:), validation error, return failure.
        if buffer.chars().nth(pointer + 1) != Some(':') {
            return None;
        }

        // Increase pointer by 2.
        pointer += 2;

        // Increase pieceIndex by 1 and then set compress to pieceIndex.
        piece_index += 1;
        compress = Some(piece_index);
    }

    // While c is not the EOF code point:
    while pointer < buffer.len() {
        // If pieceIndex is 8, validation error, return failure.
        if piece_index == 8 {
            return None;
        }

        // If c is U+003A (:), then:
        if buffer.chars().nth(pointer) == Some(':') {
            // If compress is non-null, validation error, return failure.
            if compress.is_some() {
                return None;
            }

            // Increase pointer and pieceIndex by 1, set compress to pieceIndex, and then continue.
            pointer += 1;
            piece_index += 1;
            compress = Some(piece_index);
            continue;
        }

        // Let value and length be 0.
        let mut value: u32 = 0;
        let mut length = 0;

        // While length is less than 4 and c is an ASCII hex digit, set value to value × 0x10 + c interpreted as hexadecimal number, and increase pointer and length by 1.
        while length < 4
            && pointer < buffer.len()
            && buffer.chars().nth(pointer).unwrap().is_ascii_hexdigit()
        {
            value = (value * 0x10)
                + u32::from_str_radix(
                    buffer.chars().nth(pointer).unwrap().to_string().as_str(),
                    16,
                )
                .unwrap();
            pointer += 1;
            length += 1;
        }

        // If c is U+002E (.), then:
        if buffer.chars().nth(pointer) == Some('.') {
            // If length is 0, validation error, return failure.
            if length == 0 {
                return None;
            }

            // Decrease pointer by length.
            pointer -= length;

            // If pieceIndex is greater than 6, validation error, return failure.
            if piece_index > 6 {
                return None;
            }

            // Let numbersSeen be 0.
            let mut numbers_seen = 0;

            // While c is not the EOF code point:
            while buffer.chars().nth(pointer).is_some() {
                // Let ipv4Piece be null.
                let mut ipv4_piece: Option<u32> = None;

                // If numbersSeen is greater than 0, then:
                if numbers_seen > 0 {
                    // If c is a U+002E (.) and numbersSeen is less than 4, then increase pointer by 1.
                    if buffer.chars().nth(pointer) == Some('.') && numbers_seen < 4 {
                        pointer += 1;
                    }
                    // Otherwise, validation error, return failure.
                    else {
                        return None;
                    }
                }

                // If c is not an ASCII digit, validation error, return failure.
                if let Some(c) = buffer.chars().nth(pointer) {
                    if !c.is_ascii_digit() {
                        return None;
                    }
                } else {
                    return None;
                }

                // While c is an ASCII digit:
                while let Some(c) = buffer.chars().nth(pointer) {
                    if !c.is_ascii_digit() {
                        break;
                    }

                    // Let number be c interpreted as decimal number.
                    let number = u32::from_str_radix(c.to_string().as_str(), 10).unwrap();

                    // If ipv4Piece is null, then set ipv4Piece to number.
                    if ipv4_piece.is_none() {
                        ipv4_piece = Some(number);
                    }
                    // Otherwise, if ipv4Piece is 0, validation error, return failure.
                    else if ipv4_piece == Some(0) {
                        return None;
                    }
                    // Otherwise, set ipv4Piece to ipv4Piece × 10 + number.
                    else {
                        ipv4_piece = Some((ipv4_piece.unwrap() * 10) + number);
                    }

                    // If ipv4Piece is greater than 255, validation error, return failure.
                    if ipv4_piece > Some(255) {
                        return None;
                    }

                    // Increase pointer by 1.
                    pointer += 1;
                }

                // Set address[pieceIndex] to address[pieceIndex] × 0x100 + ipv4Piece.
                address[piece_index] = (address[piece_index] * 0x100) + ipv4_piece.unwrap();

                // Increase numbersSeen by 1.
                numbers_seen += 1;

                // If numbersSeen is 2 or 4, then increase pieceIndex by 1.
                if numbers_seen == 2 || numbers_seen == 4 {
                    piece_index += 1;
                }
            }

            // If numbersSeen is not 4, validation error, return failure.
            if numbers_seen != 4 {
                return None;
            }

            // Break
            break;
        }
        // Otherwise, if c is U+003A (:):
        else if buffer.chars().nth(pointer) == Some(':') {
            // Increase pointer by 1.
            pointer += 1;

            // If c is the EOF code point, validation error, return failure.
            if buffer.chars().nth(pointer).is_none() {
                return None;
            }
        }
        // Otherwise, if c is not the EOF code point, validation error, return failure.
        else if buffer.chars().nth(pointer).is_some() {
            return None;
        }

        // Set address[pieceIndex] to value.
        address[piece_index] = value;

        // Increase pieceIndex by 1.
        piece_index += 1;
    }

    // If compress is non-null, then:
    if let Some(compress) = compress {
        // Let swaps be pieceIndex − compress.
        let mut swaps = piece_index - compress;

        // Set pieceIndex to 7.
        piece_index = 7;

        // While pieceIndex is not 0 and swaps is greater than 0, swap address[pieceIndex] with address[compress + swaps − 1], and then decrease both pieceIndex and swaps by 1.
        while piece_index != 0 && swaps > 0 {
            address.swap(piece_index as usize, (compress + swaps - 1) as usize);
            piece_index -= 1;
            swaps -= 1;
        }
    }
    // Otherwise, if compress is null and pieceIndex is not 8, validation error, return failure.
    else if piece_index != 8 {
        return None;
    }

    Some(serialize_ipv6(address))
}

pub fn parse_opaque_host(buffer: String) -> String {
    // If input contains a forbidden host code point, validation error, return failure.
    let is_invalid_host_char = |c| {
        matches!(
            c,
            '\0' | '\t'
                | '\n'
                | '\r'
                | ' '
                | '#'
                | '/'
                | ':'
                | '<'
                | '>'
                | '?'
                | '@'
                | '['
                | '\\'
                | ']'
                | '^'
                | '|'
        )
    };

    if buffer.find(is_invalid_host_char).is_some() {
        return "".to_string();
    }

    utf8_percent_encode(buffer.as_str(), CONTROLS).to_string()
}

pub fn ends_with_a_number(domain: &str) -> bool {
    // Let parts be the result of strictly splitting input on U+002E (.).
    let mut parts: Vec<&str> = domain.split(".").collect();

    // If the last item in parts is the empty string, then:
    if parts.last().unwrap_or(&"").is_empty() {
        // If parts’s size is 1, then return false.
        if parts.len() == 1 {
            return false;
        }

        // Remove the last item from parts.
        parts.pop();
    }

    // Let last be the last item in parts.
    let last = parts.last();

    // If last is non-empty and contains only ASCII digits, then return true.
    if let Some(last_item) = last {
        if last_item.bytes().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }

    // If parsing last as an IPv4 number does not return failure, then return true.
    if let Some(_number) = parse_ipv4_number(domain.to_string()) {
        return true;
    }

    false
}

/// Returns empty string if fails
pub fn parse_host(buffer: String, is_not_url_special: bool) -> String {
    // If input starts with U+005B ([), then:
    if buffer.starts_with("[") {
        // If input does not end with U+005D (]), validation error, return failure.
        if !buffer.ends_with("]") {
            return "".to_string();
        }

        // Return the result of IPv6 parsing input with its leading U+005B ([) and trailing U+005D (]) removed.
        return parse_ipv6(buffer[1..buffer.len() - 1].to_string()).unwrap_or("".to_string());
    }

    // If isNotSpecial is true, then return the result of opaque-host parsing input.
    if is_not_url_special {
        return parse_opaque_host(buffer);
    }

    let ascii_domain = domain_to_ascii(buffer.as_str()).unwrap_or("".to_string());

    // If asciiDomain is failure, validation error, return failure.
    if ascii_domain.is_empty() {
        return "".to_string();
    }

    // If asciiDomain contains a forbidden domain code point, validation error, return failure.
    let is_invalid_domain_char = |c| {
        matches!(
            c,
            '\0'..='\u{001F}'
                | ' '
                | '#'
                | '%'
                | '/'
                | ':'
                | '<'
                | '>'
                | '?'
                | '@'
                | '['
                | '\\'
                | ']'
                | '^'
                | '\u{007F}'
                | '|'
        )
    };

    if ascii_domain.find(is_invalid_domain_char).is_some() {
        return "".to_string();
    }

    // If asciiDomain ends in a number, then return the result of IPv4 parsing asciiDomain.
    if ends_with_a_number(ascii_domain.as_str()) {
        return if let Some(result) = parse_ipv4(ascii_domain) {
            serialize_ipv4(result)
        } else {
            "".to_string()
        };
    }

    ascii_domain.to_string()
}
