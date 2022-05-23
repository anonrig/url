#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

use percent_encoding::{utf8_percent_encode, CONTROLS};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt;
use url_wasm::machine::URLStateMachine;

#[derive(Deserialize, Serialize, Debug)]
struct ComplianceTest {
    input: String,
    base: String,
    href: Option<String>,
    origin: Option<String>,
    protocol: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    hostname: Option<String>,
    port: Option<String>,
    pathname: Option<String>,
    search: Option<String>,
    hash: Option<String>,
    failure: Option<bool>,
}

impl fmt::Display for ComplianceTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "input: {} ({})",
            utf8_percent_encode(self.input.as_str(), CONTROLS).to_string(),
            self.base
        )
    }
}

#[datatest::data("tests/fixtures.json")]
fn sample_test(case: ComplianceTest) {
    let machine = URLStateMachine::new(case.input.as_str(), None, None, None);

    if let Some(has_failure) = case.failure {
        assert_eq!(machine.failure, has_failure);
    }
}
