use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub enum State {
    Authority,
    SchemeStart,
    Scheme,
    Host,
    NoScheme,
    Fragment,
    Relative,
    RelativeSlash,
    File,
    FileHost,
    FileSlash,
    PathOrAuthority,
    SpecialAuthorityIgnoreSlashes,
    SpecialAuthoritySlashes,
    SpecialRelativeOrAuthority,
    Query,
    Path,
    PathStart,
    OpaquePath,
    Port,
}

#[derive(Copy, Clone)]
pub enum Code {
    Failure,
    Exit,
}

lazy_static! {
    pub static ref SPECIAL_SCHEMES: HashMap<&'static str, Option<String>> = HashMap::from([
        ("ftp", Some("21".to_string())),
        ("file", None),
        ("http", Some("80".to_string())),
        ("https", Some("443".to_string())),
        ("ws", Some("80".to_string())),
        ("wss", Some("443".to_string())),
    ]);
}
