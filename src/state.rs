use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Copy, Clone)]
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
    pub static ref SPECIAL_SCHEMES: HashMap<&'static str, Option<u32>> = HashMap::from([
        ("ftp", Some(21)),
        ("file", None),
        ("http", Some(80)),
        ("https", Some(443)),
        ("ws", Some(80)),
        ("wss", Some(443)),
    ]);
}
