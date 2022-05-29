use crate::radix::Radix;
use std::borrow::Borrow;

pub fn serialize_ipv4(address: u32) -> String {
    let mut output: String = String::from("");
    let mut n: f32 = address as f32;

    for i in 1..4 {
        output = (n % 256.0).to_string() + output.borrow();

        if i != 4 {
            output = ".".to_string() + output.borrow();
        }

        n = (n / 256.0).floor();
    }

    output.to_string()
}

pub fn serialize_ipv6(address: Vec<u32>) -> String {
    let mut output = "".to_string();
    let compress = find_longest_zero_sequence(&address);
    let mut ignore_0 = false;

    for piece_index in 0..7 {
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

        output += Radix::new(address[piece_index] as i32, 16)
            .unwrap()
            .to_string()
            .as_str();

        if piece_index != 7 {
            output += ":";
        }
    }

    output
}

fn find_longest_zero_sequence(address: &Vec<u32>) -> usize {
    let mut max_idx = 0;
    let mut max_length = 1;
    let mut current_start: Option<usize> = None;
    let mut current_length = 0;

    for (index, character) in address.into_iter().enumerate() {
        if character != &0 {
            if current_length > max_length {
                if let Some(start) = current_start {
                    max_idx = start;
                }
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

    max_idx
}
