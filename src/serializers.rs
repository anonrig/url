use std::borrow::Borrow;

pub fn serialize_ipv4(address: u64) -> String {
    let mut output: String = String::from("");
    let mut n = address as f64;

    for i in 1..=4 {
        output = (n % 256.0).to_string() + output.borrow();

        if i != 4 {
            output = ".".to_string() + output.borrow();
        }

        n = (n / 256.0).floor();
    }

    output
}

pub fn serialize_ipv6(address: Vec<u32>) -> String {
    let mut output = "".to_string();
    let compress = find_longest_zero_sequence(&address);
    let mut ignore_0 = false;

    for piece_index in 0..8 {
        if ignore_0 && address[piece_index] == 0 {
            continue;
        } else if ignore_0 {
            ignore_0 = false;
        }

        if compress == piece_index {
            output += if piece_index == 0 { "::" } else { ":" };
            ignore_0 = true;
            continue;
        }

        output += char::from_u32(address[piece_index])
            .unwrap()
            .to_digit(16)
            .unwrap_or(0)
            .to_string()
            .as_str();

        if piece_index != 7 {
            output += ":";
        }
    }

    output
}

fn find_longest_zero_sequence(address: &Vec<u32>) -> usize {
    let mut max_idx: Option<usize> = None;
    let mut max_length = 1;
    let mut current_start: Option<usize> = None;
    let mut current_length = 0;

    for (index, character) in address.iter().enumerate() {
        if character != &0 {
            if current_length > max_length {
                max_idx = current_start;
                max_length = current_length;
            }

            current_start = None;
            current_length = 0;
        } else {
            if current_start.is_none() {
                current_start = Some(index);
            }

            current_length += 1;
        }
    }

    if current_length > max_length {
        return current_start.unwrap_or(0);
    }

    max_idx.unwrap_or(0)
}
