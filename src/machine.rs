use crate::encode_sets::{
    FRAGMENT_PERCENT_ENCODE_SET, PATH_PERCENT_ENCODE_SET, QUERY_PERCENT_ENCODE_SET,
    SPECIAL_QUERY_PERCENT_ENCODE_SET, USER_INFO_PERCENT_ENCODE_SET,
};
use crate::parser::parse_host;
use crate::platform::{
    is_normalized_windows_drive_letter, is_windows_drive_letter, starts_with_windows_drive_letter,
};
use crate::state::{Code, State, SPECIAL_SCHEMES};
use crate::string::{
    is_ascii_alphanumeric, is_ascii_digit, is_double_dot_path_segment, is_single_dot_path_segment,
};
use crate::url::URL;
use percent_encoding::{utf8_percent_encode, CONTROLS};
use std::borrow::Borrow;

pub struct URLStateMachine {
    buffer: String,
    at_sign_seen: bool,
    inside_brackets: bool,
    password_token_seen: bool,
    pointer: i32,
    pub failure: bool,
    state_override: bool,
    encoding_override: String,
    is_special_url: bool,
    state: State,
    base: Option<URL>,
    pub url: URL,
    input: String,
}

impl URLStateMachine {
    pub fn new(
        input: &str,
        base: Option<URL>,
        encoding_override: Option<String>,
        state_override: Option<State>,
    ) -> URLStateMachine {
        // If input contains any leading or trailing C0 control or space, validation error.
        // If input contains any ASCII tab or newline, validation error.
        let trimmed_input = input
            .trim_matches(|c| c <= ' ')
            .replace(|c| c == '\t' || c == '\n' || c == '\r', "")
            .to_string();

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
            input: trimmed_input.clone(),
        };

        let mut bytes: Vec<Option<u8>> = trimmed_input.bytes().map(|c| Some(c)).collect();

        // Traverse one more time for EOL character.
        bytes.push(None);

        while machine.pointer < bytes.len() as i32 {
            let byte = bytes[machine.pointer as usize];

            let result = match machine.state {
                State::Authority => machine.authority_state(byte),
                State::SchemeStart => machine.scheme_start_state(byte),
                State::Scheme => machine.scheme_state(byte),
                State::Host => machine.host_state(byte),
                State::NoScheme => machine.no_scheme_state(byte),
                State::Fragment => machine.fragment_state(byte),
                State::Relative => machine.relative_state(byte),
                State::RelativeSlash => machine.relative_slash_state(byte),
                State::File => machine.file_state(byte),
                State::FileHost => machine.file_host_state(byte),
                State::FileSlash => machine.file_slash_state(byte),
                State::PathOrAuthority => machine.path_or_authority_state(byte),
                State::SpecialAuthorityIgnoreSlashes => {
                    machine.special_authority_ignore_slashes_state(byte)
                }
                State::SpecialAuthoritySlashes => machine.special_authority_slashes_state(byte),
                State::SpecialRelativeOrAuthority => {
                    machine.special_relative_or_authority_state(byte)
                }
                State::Query => machine.query_state(byte),
                State::Path => machine.path_state(byte),
                State::PathStart => machine.path_start_state(byte),
                State::OpaquePath => machine.opaque_path_state(byte),
                State::Port => machine.port_state(byte),
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

            machine.pointer += 1;
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
            self.buffer += (code.unwrap() as char).to_lowercase().to_string().as_str();
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

    fn scheme_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is an ASCII alphanumeric, U+002B (+), U+002D (-), or U+002E (.), append c, lowercased, to buffer.
        if is_ascii_alphanumeric(code) || code == Some(43) || code == Some(45) || code == Some(46) {
            self.buffer += (code.unwrap() as char).to_lowercase().to_string().as_str();
        }
        // Otherwise, if c is U+003A (:), then:
        else if code == Some(58) {
            let is_buffer_special = SPECIAL_SCHEMES.contains_key(self.buffer.as_str());

            // If state override is given, then:
            if self.state_override {
                // If url’s scheme is a special scheme and buffer is not a special scheme, then return.
                // If url’s scheme is not a special scheme and buffer is a special scheme, then return.
                // If url includes credentials or has a non-null port, and buffer is 'file', then return.
                // If url’s scheme is 'file' and its host is an empty host, then return.
                if (self.is_special_url && !is_buffer_special)
                    || (!self.is_special_url && is_buffer_special)
                    || ((self.url.username.len() > 0
                        || self.url.password.len() > 0
                        || self.url.port.is_some())
                        && self.buffer == *"file")
                    || (self.url.scheme == *"file"
                        && self.url.host.is_some()
                        && self.url.host.as_ref().unwrap().len() == 0)
                {
                    return Some(Code::Exit);
                }
            }

            // Set url’s scheme to buffer.
            self.is_special_url = is_buffer_special;
            self.url.scheme = self.buffer.clone();

            // If state override is given, then:
            if self.state_override {
                let port = SPECIAL_SCHEMES
                    .get(self.url.scheme.as_str())
                    .unwrap()
                    .as_ref();

                // If url’s port is url’s scheme’s default port, then set url’s port to null.
                // TODO: Fix this
                if let Some(port) = port {
                    if self.url.port == Some(port.to_string()) {
                        self.url.port = None;
                        return Some(Code::Exit);
                    }
                }
            }

            // Set buffer to the empty string.
            self.buffer = "".to_string();

            // If url’s scheme is "file", then:
            if self.url.scheme == *"file" {
                // Set state to file state.
                self.state = State::File;
            }
            // Otherwise, if url is special, base is non-null, and base’s scheme is url’s scheme:
            else if self.is_special_url
                && self.base.is_some()
                && self.base.as_ref().unwrap().scheme == self.url.scheme
            {
                // Set state to special relative or authority state.
                self.state = State::SpecialRelativeOrAuthority;
            }
            // Otherwise, if url is special, set state to special authority slashes state.
            else if self.is_special_url {
                self.state = State::SpecialAuthoritySlashes;
            }
            // Otherwise, if remaining starts with an U+002F (/), set state to path or authority state and increase pointer by 1.
            else if self.input.chars().nth(self.pointer as usize + 1) == Some('/') {
                self.state = State::PathOrAuthority;
                self.pointer += 1;
            }
            // Otherwise, set url’s path to the empty string and set state to opaque path state.
            else {
                self.url.path = vec![];
                self.state = State::OpaquePath;
            }
        }
        // Otherwise, if state override is not given, set buffer to the empty string, state to no scheme state,
        // and start over (from the first code point in input).
        else if !self.state_override {
            self.buffer = "".to_string();
            self.state = State::NoScheme;
            self.pointer = -1;
        }
        // Otherwise, validation error, return failure.
        else {
            return Some(Code::Failure);
        }

        None
    }

    fn host_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If state override is given and url’s scheme is "file", then decrease pointer by 1 and set state to file host state.
        if self.state_override && self.url.scheme == *"file" {
            self.pointer -= 1;
            self.state = State::FileHost;
        }
        // Otherwise, if c is U+003A (:) and insideBrackets is false, then:
        else if code == Some(58) && !self.inside_brackets {
            // If buffer is the empty string, validation error, return failure.
            if self.buffer.is_empty() {
                return Some(Code::Failure);
            }

            // If state override is given and state override is hostname state, then return.
            // TODO: Implement this by changing state_override type from bool to Option<State>

            // Let host be the result of host parsing buffer with url is not special.
            let host = parse_host(self.buffer.clone(), !self.is_special_url);

            // If host is failure, then return failure.
            if host.is_empty() {
                return Some(Code::Failure);
            }

            // Set url’s host to host, buffer to the empty string, and state to port state.
            self.url.host = Some(host);
            self.buffer = "".to_string();
            self.state = State::Port;
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
            // then decrease pointer by 1, and then:
            self.pointer -= 1;

            // If url is special and buffer is the empty string, validation error, return failure.
            if self.is_special_url && self.buffer.is_empty() {
                return Some(Code::Failure);
            }
            // Otherwise, if state override is given, buffer is the empty string, and either url includes credentials or url’s port is non-null, return.
            else if self.state_override
                && self.buffer.is_empty()
                && (self.url.port.is_some()
                    || !self.url.username.is_empty()
                    || !self.url.password.is_empty())
            {
                return Some(Code::Exit);
            }

            let host = parse_host(self.buffer.clone(), !self.is_special_url);

            if host.is_empty() {
                return Some(Code::Failure);
            }

            // Set url’s host to host, buffer to the empty string, and state to path start state.
            self.url.host = Some(host);
            self.buffer = "".to_string();
            self.state = State::PathStart;

            // If state override is given, then return.
            if self.state_override {
                return Some(Code::Exit);
            }
        } else if let Some(code) = code {
            // If c is U+005B ([), then set insideBrackets to true.
            if code == 91 {
                self.inside_brackets = true;
            }
            // If c is U+005D (]), then set insideBrackets to false.
            else if code == 93 {
                self.inside_brackets = false;
            }

            self.buffer += (code as char).to_string().as_str();
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
                let input = self.input.chars().nth(self.pointer as usize).unwrap();

                let encoded_code_points =
                    utf8_percent_encode(input.to_string().as_str(), USER_INFO_PERCENT_ENCODE_SET)
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
            self.pointer -= (self.buffer.len() + 1) as i32;
            self.buffer = "".to_string();
            self.state = State::Host;
        }
        // Otherwise, append c to buffer.
        else if let Some(c) = self.input.chars().nth(self.pointer as usize) {
            self.buffer += c.to_string().as_str();
        }

        None
    }

    fn fragment_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If c is not the EOF code point, then:
        if let Some(_code) = code {
            let fragment = &self.input[self.pointer as usize..];
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
        if code == Some(47) && self.input.chars().nth(self.pointer as usize + 1) == Some('/') {
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
        if code == Some(47) && self.input.chars().nth(self.pointer as usize + 1) == Some('/') {
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
                if let Some(c) = self.input.chars().nth(self.pointer as usize) {
                    self.url
                        .path
                        .push(utf8_percent_encode(c.to_string().as_str(), CONTROLS).to_string());
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
            self.url.port = base.port.clone();
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
            self.url.port = base.port.clone();
            self.state = State::Path;
            self.pointer -= 1;
        }

        None
    }

    fn path_start_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If url is special, then:
        if self.is_special_url {
            // Set state to path state.
            self.state = State::Path;

            // If c is neither U+002F (/) nor U+005C (\), then decrease pointer by 1.
            if code != Some(47) && code != Some(92) {
                self.pointer -= 1;
            }
        }
        // Otherwise, if state override is not given and c is U+003F (?), set url’s query to the empty string and state to query state.
        else if !self.state_override && code == Some(63) {
            self.url.query = Some("".to_string());
            self.state = State::Query;
        }
        // Otherwise, if state override is not given and c is U+0023 (#), set url’s fragment to the empty string and state to fragment state.
        else if !self.state_override && code == Some(35) {
            self.state = State::Fragment;
        }
        // Otherwise, if c is not the EOF code point:
        else if code.is_some() {
            // Set state to path state.
            self.state = State::Path;

            // If c is not U+002F (/), then decrease pointer by 1.
            if code != Some(47) {
                self.pointer -= 1;
            }
        }
        // Otherwise, if state override is given and url’s host is null, append the empty string to url’s path.
        else if self.state_override && self.url.host.is_none() {
            self.url.path.push("/".to_string());
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
                let port_value = u16::from_str_radix(self.buffer.as_str(), 10);

                // If port is greater than 2^16 − 1, validation error, return failure.
                if port_value.is_err() {
                    return Some(Code::Failure);
                }

                let port = port_value.unwrap().to_string();

                // Set url’s port to null, if port is url’s scheme’s default port; otherwise to port.
                if let Some(default_port_unwrapped) = SPECIAL_SCHEMES.get(self.url.scheme.as_str())
                {
                    if default_port_unwrapped.is_some() {
                        let default_port = default_port_unwrapped.as_ref().unwrap().to_string();
                        self.url.port = if port == default_port {
                            None
                        } else {
                            Some(port)
                        };
                    } else {
                        self.url.port = Some(port);
                    }
                } else {
                    self.url.port = Some(port);
                }

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

    fn query_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If encoding is not UTF-8 and one of the following is true:
        // - url is not special
        // - url’s scheme is "ws" or "wss"
        // then set encoding to UTF-8.
        if self.encoding_override != "utf-8"
            && (!self.is_special_url || self.url.scheme == *"ws" || self.url.scheme == *"wss")
        {
            self.encoding_override = "utf-8".to_string();
        }

        // If one of the following is true:
        // - state override is not given and c is U+0023 (#)
        // - c is the EOF code point
        if (!self.state_override && code == Some(35)) || code.is_none() {
            // Let queryPercentEncodeSet be the special-query percent-encode set if url is special; otherwise the query percent-encode set.
            let encoding_set = if self.is_special_url {
                SPECIAL_QUERY_PERCENT_ENCODE_SET
            } else {
                QUERY_PERCENT_ENCODE_SET
            };

            // Percent-encode after encoding, with encoding, buffer, and queryPercentEncodeSet, and append the result to url’s query.
            if let Some(query) = self.url.query.clone() {
                self.url.query = Some(
                    query
                        + utf8_percent_encode(self.buffer.as_str(), encoding_set)
                            .to_string()
                            .borrow(),
                );
            }

            // Set buffer to the empty string.
            self.buffer = "".to_string();

            // If c is U+0023 (#), then set url’s fragment to the empty string and state to fragment state.
            if code == Some(35) {
                self.state = State::Fragment;
            }
        }
        // Otherwise, if c is not the EOF code point: Append c to buffer
        else if let Some(c) = self.input.chars().nth(self.pointer as usize) {
            self.buffer += c.to_string().as_str()
        }

        None
    }

    fn path_state(&mut self, code: Option<u8>) -> Option<Code> {
        // If one of the following is true:
        // - c is the EOF code point or U+002F (/)
        // - url is special and c is U+005C (\)
        // - state override is not given and c is U+003F (?) or U+0023 (#)
        if code.is_none()
            || code == Some(47)
            || (self.is_special_url && code == Some(92))
            || (!self.state_override && (code == Some(63)) || code == Some(35))
        {
            // If buffer is a double-dot path segment, then:
            if is_double_dot_path_segment(self.buffer.to_lowercase().as_str()) {
                // Shorten url’s path.
                self.shorten_url();

                // If neither c is U+002F (/), nor url is special and c is U+005C (\), append the empty string to url’s path.
                if code != Some(47) && !(self.is_special_url && code == Some(92)) {
                    self.url.path.push("".to_string());
                }
            }
            // Otherwise, if buffer is a single-dot path segment and if neither c is U+002F (/),
            // nor url is special and c is U+005C (\), append the empty string to url’s path.
            else if is_single_dot_path_segment(self.buffer.as_str())
                && code != Some(47)
                && !(self.is_special_url && code == Some(92))
            {
                self.url.path.push("".to_string());
            }
            // Otherwise, if buffer is not a single-dot path segment, then:
            else if !is_single_dot_path_segment(self.buffer.as_str()) {
                // If url’s scheme is "file", url’s path is empty, and buffer is a Windows drive letter,
                // then replace the second code point in buffer with U+003A (:).
                if self.url.scheme == *"file"
                    && self.url.path.is_empty()
                    && is_windows_drive_letter(self.buffer.as_str())
                {
                    self.buffer = self.buffer.chars().next().unwrap().to_string() + ":";
                }

                // Append buffer to url’s path.
                self.url.path.push(self.buffer.clone());
            }

            // Set buffer to the empty string.
            self.buffer = "".to_string();

            // If c is U+003F (?), then set url’s query to the empty string and state to query state.
            if code == Some(63) {
                self.url.query = Some("".to_string());
                self.state = State::Query;
            } else if code == Some(35) {
                // If c is U+0023 (#), then set url’s fragment to the empty string and state to fragment state.
                self.url.fragment = Some("".to_string());
                self.state = State::Fragment;
            }
        }
        // Otherwise run these steps:
        else if let Some(c) = self.input.chars().nth(self.pointer as usize) {
            self.buffer += utf8_percent_encode(c.to_string().as_str(), PATH_PERCENT_ENCODE_SET)
                .to_string()
                .as_str();
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
            if let Some(base) = self.base.as_ref() {
                if base.scheme == *"file" {
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
            }

            // Set state to path state, and decrease pointer by 1.
            self.state = State::Path;
            self.pointer -= 1;
        }

        None
    }

    fn file_state(&mut self, code: Option<u8>) -> Option<Code> {
        // Set url's scheme to "file".
        self.url.scheme = "file".to_string();
        self.is_special_url = true;

        // Set url’s host to the empty string.
        self.url.host = Some("".to_string());

        // If c is U+002F (/) or U+005C (\), then:
        if code == Some(47) || code == Some(92) {
            // Set state to file slash state.
            self.state = State::FileSlash;
        }
        // Otherwise, if base is non-null and base’s scheme is "file":
        else if self.base.is_some() && self.base.as_ref().unwrap().scheme == *"file" {
            let base = self.base.as_ref().unwrap();

            // Set url’s host to base’s host, url’s path to a clone of base’s path, and url’s query to base’s query.
            self.url.host = base.host.clone();
            self.url.path = base.path.clone();
            self.url.query = base.query.clone();

            // If c is U+003F (?), then set url’s query to the empty string and state to query state.
            if code == Some(63) {
                self.url.query = Some("".to_string());
                self.state = State::Query;
            }
            // Otherwise, if c is U+0023 (#), set url’s fragment to the empty string and state to fragment state.
            else if code == Some(35) {
                self.url.fragment = Some("".to_string());
                self.state = State::Fragment;
            }
            // Otherwise, if c is not the EOF code point:
            else if code.is_some() {
                // Set url’s query to null.
                self.url.query = None;

                // If the code point substring from pointer to the end of input does not start with a Windows drive letter, then shorten url’s path.
                if !starts_with_windows_drive_letter(self.input.as_str(), self.pointer as usize) {
                    self.shorten_url();
                }
                // Otherwise:
                else {
                    // Set url's path to an empty list.
                    self.url.path = vec![];
                }

                // Set state to path state and decrease pointer by 1.
                self.state = State::Path;
                self.pointer -= 1;
            }
        }
        // Otherwise, set state to path state, and decrease pointer by 1.
        else {
            self.state = State::Path;
            self.pointer -= 1;
        }

        None
    }

    fn file_host_state(&mut self, code: Option<u8>) -> Option<Code> {
        if let Some(wrapped_code) = code {
            self.buffer += (wrapped_code as char).to_string().as_str()
        }
        // If c is the EOF code point, U+002F (/), U+005C (\), U+003F (?), or U+0023 (#), then decrease pointer by 1 and then:
        else if code.is_none()
            || code == Some(47)
            || code == Some(92)
            || code == Some(63)
            || code == Some(35)
        {
            self.pointer -= 1;

            // If state override is not given and buffer is a Windows drive letter, validation error, set state to path state.
            if !self.state_override && is_windows_drive_letter(self.buffer.as_str()) {
                self.state = State::Path;
            }
            // Otherwise, if buffer is the empty string, then:
            else if self.buffer.is_empty() {
                // Set url’s host to the empty string.
                self.url.host = Some("".to_string());

                // If state override is given, then return.
                if self.state_override {
                    return Some(Code::Exit);
                }

                // Set state to path start state.
                self.state = State::PathStart;
            }
            // Otherwise, run these steps:
            else {
                // Let host be the result of host parsing buffer with url is not special.
                let mut host = parse_host(self.buffer.clone(), !self.is_special_url);

                // If host is failure, then return failure.
                if host.is_empty() {
                    return Some(Code::Failure);
                }

                // If host is "localhost", then set host to the empty string.
                if host == *"localhost" {
                    host = "".to_string();
                }

                // Set url’s host to host.
                self.url.host = Some(host);

                // If state override is given, then return.
                if self.state_override {
                    return Some(Code::Exit);
                }

                // Set buffer to the empty string and state to path start state.
                self.buffer = "".to_string();
                self.state = State::PathStart;
            }
        }

        None
    }
}
