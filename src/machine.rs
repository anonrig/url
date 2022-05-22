use crate::encode_sets::{FRAGMENT_PERCENT_ENCODE_SET, USER_INFO_PERCENT_ENCODE_SET};
use crate::state::{Code, State};
use crate::url::URL;
use percent_encoding::utf8_percent_encode;
use std::borrow::Borrow;
use std::str::from_utf8;

pub struct URLStateMachine {
    buffer: String,
    at_sign_seen: bool,
    inside_brackets: bool,
    password_token_seen: bool,
    pointer: usize,
    failure: bool,
    encoding_override: String,
    is_special_url: bool,
    state: State,
    base: Option<URL>,
    url: URL,
    input: String,
}

impl URLStateMachine {
    pub fn new(
        input: &str,
        base: Option<URL>,
        encoding_override: Option<String>,
        state_override: Option<State>,
    ) -> URLStateMachine {
        let mut machine = URLStateMachine {
            buffer: "".to_string(),
            at_sign_seen: false,
            inside_brackets: false,
            password_token_seen: false,
            pointer: 0,
            failure: false,
            encoding_override: encoding_override.unwrap_or_else(|| "utf-8".to_string()),
            is_special_url: false,
            state: state_override.unwrap_or(State::SchemeStart),
            base,
            url: URL::new(),
            input: input.to_string(),
        };

        for byte in input.bytes() {
            let result = match machine.state {
                State::Authority => machine.authority_state(Some(byte)),
                State::SchemeStart => None,
                State::Scheme => None,
                State::Host => None,
                State::NoScheme => None,
                State::Fragment => machine.fragment_state(Some(byte)),
                State::Relative => None,
                State::RelativeSlash => None,
                State::File => None,
                State::FileHost => None,
                State::FileSlash => None,
                State::PathOrAuthority => machine.path_or_authority_state(Some(byte)),
                State::SpecialAuthorityIgnoreSlashes => {
                    machine.special_authority_ignore_slashes_state(Some(byte))
                }
                State::SpecialAuthoritySlashes => {
                    machine.special_authority_slashes_state(Some(byte))
                }
                State::SpecialRelativeOrAuthority => {
                    machine.special_relative_or_authority_state(Some(byte))
                }
                State::Query => None,
                State::Path => None,
                State::PathStart => None,
                State::OpaquePath => None,
                State::Port => None,
            };

            match result {
                None => {}
                Some(Code::Failure) => {
                    machine.failure = true;
                    break;
                }
                Some(Code::Exit) => {
                    break;
                }
            }
        }

        machine
    }
}

impl URLStateMachine {
    fn authority_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is U+0040 (@), then:
        if code == Some(64) {
            // If atSignSeen is true, then prepend '%40' to buffer.
            if self.at_sign_seen {
                self.buffer = "%40".to_string() + self.buffer.as_str();
            }

            self.at_sign_seen = true;

            // For each codePoint in buffer:
            for code_point in self.buffer.bytes() {
                // If codePoint is U+003A (:) and passwordTokenSeen is false, then set passwordTokenSeen to true and continue.
                if code_point == 58 && !self.password_token_seen {
                    self.password_token_seen = true;
                    continue;
                }

                // Let encodedCodePoints be the result of running UTF-8 percent-encode codePoint using the userinfo percent-encode set.
                let input = [code.unwrap()];
                let encoded_code_points = utf8_percent_encode(
                    from_utf8(input.borrow()).unwrap(),
                    USER_INFO_PERCENT_ENCODE_SET,
                )
                .to_string();

                // If passwordTokenSeen is true, then append encodedCodePoints to url’s password.
                if self.password_token_seen {
                    self.url.password += encoded_code_points.as_str();
                } else {
                    // Otherwise, append encodedCodePoints to url’s username.
                    self.url.username += encoded_code_points.as_str();
                }
            }

            self.buffer = "".to_string()
        }
        // Otherwise, if one of the following is true:
        // - c is the EOF code point, U+002F (/), U+003F (?), or U+0023 (#)
        // - url is special and c is U+005C (\)
        else if code.is_none()
            || code == Some(47)
            || code == Some(63)
            || code == Some(35)
            || (self.is_special_url && code == Some(92))
        {
            // If atSignSeen is true and buffer is the empty string, validation error, return failure.
            if self.at_sign_seen && self.buffer.len() == 1 {
                return Some(Code::Failure);
            }

            // Decrease pointer by the number of code points in buffer plus one, set buffer to the empty string, and set state to host state.
            self.pointer -= self.buffer.len() + 1;
            self.buffer = "".to_string();
            self.state = State::Host;
        }
        // Otherwise, append c to buffer.
        else {
            self.buffer
                .push_str(from_utf8(vec![code.unwrap()].borrow()).unwrap());
        }

        None
    }

    fn fragment_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is not the EOF code point, then:
        if let Some(_code) = code {
            let fragment = &self.input[self.pointer..self.input.len()];
            self.url.fragment =
                Some(utf8_percent_encode(fragment, FRAGMENT_PERCENT_ENCODE_SET).to_string());
        }

        Some(Code::Exit)
    }

    fn special_authority_ignore_slashes_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is neither U+002F (/) nor U+005C (\), then set state to authority state and decrease pointer by 1.
        if code != Some(47) && code != Some(92) {
            self.state = State::Authority;
            self.pointer -= 1;
        }

        None
    }

    fn special_authority_slashes_state(&mut self, code: Option<u8>) -> Option<Code> {
        self.state = State::SpecialAuthorityIgnoreSlashes;

        // If c is U+002F (/) and remaining starts with U+002F (/),
        if code == Some(47) && self.input.chars().nth(self.pointer + 1) == Some('/') {
            // then set state to special authority ignore slashes state and increase pointer by 1.
            self.pointer += 1;
        } else {
            // Otherwise, validation error, set state to special authority ignore slashes state and decrease pointer by 1.
            self.pointer -= 1;
        }

        None
    }

    fn special_relative_or_authority_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is U+002F (/) and remaining starts with U+002F (/),
        // then set state to special authority ignore slashes state and increase pointer by 1.
        if code == Some(47) && self.input.chars().nth(self.pointer + 1) == Some('/') {
            self.state = State::SpecialAuthorityIgnoreSlashes;
            self.pointer += 1;
        }
        // // Otherwise, validation error, set state to relative state and decrease pointer by 1.
        else {
            self.state = State::Relative;
            self.pointer -= 1;
        }

        None
    }

    fn path_or_authority_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is U+002F (/), then set state to authority state.
        if code == Some(47) {
            self.state = State::Authority;
        }
        // Otherwise, set state to path state, and decrease pointer by 1.
        else {
            self.state = State::Path;
            self.pointer -= 1;
        }

        None
    }
}