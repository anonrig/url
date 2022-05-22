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

pub enum Code {
    Failure,
    Exit,
}
