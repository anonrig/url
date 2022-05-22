use crate::encode_sets::{FRAGMENT_PERCENT_ENCODE_SET, USER_INFO_PERCENT_ENCODE_SET};
use crate::platform::{is_normalized_windows_drive_letter, starts_with_windows_drive_letter};
use crate::state::{Code, State, SPECIAL_SCHEMES};
use crate::string::{is_ascii_alphanumeric, is_ascii_digit};
use crate::url::URL;
use percent_encoding::{utf8_percent_encode, CONTROLS};
use std::borrow::Borrow;
use std::str::from_utf8;

pub struct URLStateMachine {
    buffer: String,
    at_sign_seen: bool,
    inside_brackets: bool,
    password_token_seen: bool,
    pointer: usize,
    failure: bool,
    state_override: bool,
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
            state_override: state_override.is_some(),
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
                State::SchemeStart => machine.scheme_start_state(Some(byte)),
                State::Scheme => None,
                State::Host => None,
                State::NoScheme => machine.no_scheme_state(Some(byte)),
                State::Fragment => machine.fragment_state(Some(byte)),
                State::Relative => machine.relative_state(Some(byte)),
                State::RelativeSlash => machine.relative_slash_state(Some(byte)),
                State::File => None,
                State::FileHost => None,
                State::FileSlash => machine.file_slash_state(Some(byte)),
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
                State::OpaquePath => machine.opaque_path_state(Some(byte)),
                State::Port => machine.port_state(Some(byte)),
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
    fn shorten_url(&mut self) {
        // If url’s scheme is "file", path’s size is 1, and path[0] is a normalized Windows drive letter, then return.
        if self.url.scheme == *"file"
            && self.url.path.len() == 1
            && is_normalized_windows_drive_letter(self.url.path.first().unwrap())
        {
            return;
        }

        self.url.path.pop();
    }
}

impl URLStateMachine {
    fn scheme_start_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is an ASCII alpha, append c, lowercased, to buffer, and set state to scheme state.
        if is_ascii_alphanumeric(code) {
            self.buffer
                .push_str((code.unwrap() as char).to_lowercase().to_string().as_str());
            self.state = State::Scheme;
        }
        // Otherwise, if state override is not given, set state to no scheme state and decrease pointer by 1.
        else if !self.state_override {
            self.state = State::NoScheme;
            self.pointer -= 1;
        }
        // Otherwise, validation error, return failure.
        else {
            return Some(Code::Failure);
        }

        None
    }

    fn no_scheme_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If base is null, or base has an opaque path and c is not U+0023 (#), validation error, return failure.
        // TODO: Handle opaque path
        if self.base.is_none() || code != Some(35) {
            return Some(Code::Failure);
        }

        let base = self.base.as_ref().unwrap();

        // Otherwise, if base has an opaque path and c is U+0023 (#), set url’s scheme to base’s scheme,
        // url’s path to base’s path, url’s query to base’s query, url’s fragment to the empty string,
        // and set state to fragment state.
        if code == Some(35) {
            self.is_special_url = SPECIAL_SCHEMES.contains_key(base.scheme.as_str());
            self.url.scheme = base.scheme.clone();
            self.url.path = base.path.clone();
            self.url.query = base.query.clone();
            self.url.fragment = Some("".to_string());
        }
        // Otherwise, if base’s scheme is not 'file', set state to relative state and decrease pointer by 1.
        else if base.scheme != "file".to_lowercase() {
            self.state = State::Relative;
            self.pointer -= 1;
        }
        // Otherwise, set state to file state and decrease pointer by 1.
        else {
            self.state = State::File;
            self.pointer -= 1;
        }

        None
    }

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

    fn opaque_path_state(&mut self, code: Option<u8>) -> Option<Code> {
        match code {
            Some(63) => {
                // If c is U+003F (?), then set url’s query to the empty string and state to query state.
                self.url.query = Some("".to_string());
                self.state = State::Query;
            }
            Some(35) => {
                // Otherwise, if c is U+0023 (#), then set url’s fragment to the empty string and state to fragment state.
                self.state = State::Fragment;
            }
            _ => {
                // If c is not the EOF code point, UTF-8 percent-encode c using the C0 control percent-encode set and append the result to url’s path.
                if code != None {
                    let input = [code.unwrap()];
                    self.url.path.push(
                        utf8_percent_encode(from_utf8(input.borrow()).unwrap(), CONTROLS)
                            .to_string(),
                    );
                }
            }
        }

        None
    }

    fn relative_state(&mut self, code: Option<u8>) -> Option<Code> {
        // Set url’s scheme to base’s scheme.
        let base = self.base.as_ref().unwrap();

        self.is_special_url = SPECIAL_SCHEMES.contains_key(base.scheme.as_str());
        self.url.scheme = base.scheme.clone();

        // If c is U+002F (/), then set state to relative slash state.
        // Otherwise, if url is special and c is U+005C (\), validation error, set state to relative slash state.
        if code == Some(47) || (self.is_special_url && code == Some(92)) {
            self.state = State::RelativeSlash;
        }
        // Otherwise:
        else {
            // Set url’s username to base’s username, url’s password to base’s password, url’s host to base’s host,
            // url’s port to base’s port, url’s path to a clone of base’s path, and url’s query to base’s query.
            self.url.username = base.username.clone();
            self.url.password = base.password.clone();
            self.url.host = base.host.clone();
            self.url.port = base.port;
            self.url.path = base.path.clone();
            self.url.query = base.query.clone();

            // If c is U+003F (?), then set url’s query to the empty string, and state to query state.
            if code == Some(63) {
                self.url.query = Some("".to_string());
                self.state = State::Query;
            }
            // Otherwise, if c is U+0023 (#), set url’s fragment to the empty string and state to fragment state.
            else if code == Some(35) {
                self.url.fragment = Some("".to_string());
                self.state = State::Fragment;
            }
            // Otherwise, if c is not the EOF code point
            else if code.is_some() {
                self.url.query = None;
                self.shorten_url();
                self.state = State::Path;
                self.pointer -= 1;
            }
        }

        None
    }

    fn relative_slash_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If url is special and c is U+002F (/) or U+005C (\), then:
        if (self.is_special_url && code == Some(47)) || code == Some(92) {
            // Set state to special authority ignore slashes state.
            self.state = State::SpecialAuthorityIgnoreSlashes
        }
        // Otherwise, if c is U+002F (/), then set state to authority state.
        else if code == Some(47) {
            self.state = State::Authority;
        }
        // Otherwise, set url’s username to base’s username, url’s password to base’s password, url’s host to base’s host,
        // url’s port to base’s port, state to path state, and then, decrease pointer by 1.
        else {
            let base = self.base.as_ref().unwrap();
            self.url.username = base.username.clone();
            self.url.password = base.password.clone();
            self.url.host = base.host.clone();
            self.url.port = base.port;
            self.state = State::Path;
            self.pointer -= 1;
        }

        None
    }

    fn port_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is an ASCII digit, append c to buffer.
        if is_ascii_digit(code) {
            self.buffer.push(code.unwrap() as char);
        }
        // Otherwise, if one of the following is true:
        // - c is the EOF code point, U+002F (/), U+003F (?), or U+0023 (#)
        // - url is special and c is U+005C (\)
        // - state override is given
        else if code.is_none()
            || code == Some(47)
            || code == Some(63)
            || code == Some(35)
            || (self.is_special_url && code == Some(92))
            || self.state_override
        {
            // If buffer is not the empty string, then:
            if !self.buffer.is_empty() {
                // Let port be the mathematical integer value that is represented by buffer in radix-10 using ASCII digits for digits with values 0 through 9.
                let port = self.buffer.parse::<u32>().unwrap();

                // If port is greater than 2^16 − 1, validation error, return failure.
                if port > (2_i32.pow(16) - 1).try_into().unwrap() {
                    return Some(Code::Failure);
                }

                // Set url’s port to null, if port is url’s scheme’s default port; otherwise to port.
                let default_port = SPECIAL_SCHEMES.get(self.url.scheme.as_str()).unwrap();

                self.url.port = if default_port == Some(port).borrow() {
                    None
                } else {
                    Some(port)
                };

                // Set buffer to the empty string.
                self.buffer = "".to_string();
            }
            // If state override is given, then return.
            else if self.state_override {
                return Some(Code::Exit);
            }

            // Set state to path start state and decrease pointer by 1.
            self.state = State::PathStart;
            self.pointer -= 1;
        }
        // Otherwise, validation error, return failure.
        else {
            return Some(Code::Failure);
        }

        None
    }

    fn file_slash_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is U+002F (/) or U+005C (\), then:
        if code == Some(47) || code == Some(92) {
            self.state = State::FileHost;
        }
        // Otherwise:
        else {
            // If base is non-null and base’s scheme is "file", then:
            if self.base.is_some() && self.base.as_ref().unwrap().scheme == *"file" {
                let base = self.base.as_ref().unwrap();

                // Set url’s host to base’s host.
                self.url.host = base.host.clone();

                // If the code point substring from pointer to the end of input does not start with a Windows drive
                // letter and base’s path[0] is a normalized Windows drive letter, then append base’s path[0] to url’s path.
                if !starts_with_windows_drive_letter(self.input.as_str(), self.pointer as usize)
                    && !base.path.is_empty()
                    && is_normalized_windows_drive_letter(base.path.first().unwrap())
                {
                    self.url.path.push(base.path.first().unwrap().to_string())
                }
            }

            // Set state to path state, and decrease pointer by 1.
            self.state = State::Path;
            self.pointer -= 1;
        }

        None
    }
}
