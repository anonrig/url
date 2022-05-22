use percent_encoding::{AsciiSet, CONTROLS};

/// The query percent-encode set is the C0 control percent-encode set and U+0020 SPACE, U+0022 ("), U+0023 (#), U+003C (<), and U+003E (>).
pub const QUERY_PERCENT_ENCODE_SET: &AsciiSet =
    &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');

/// The path percent-encode set is the query percent-encode set and U+003F (?), U+0060 (`), U+007B ({), and U+007D (}).
pub const PATH_PERCENT_ENCODE_SET: &AsciiSet = &QUERY_PERCENT_ENCODE_SET
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

/// The userinfo percent-encode set is the path percent-encode set and
/// U+002F (/), U+003A (:), U+003B (;), U+003D (=), U+0040 (@), U+005B ([) to U+005E (^), inclusive, and U+007C (|).
pub const USER_INFO_PERCENT_ENCODE_SET: &AsciiSet = &PATH_PERCENT_ENCODE_SET
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'|');

/// The fragment percent-encode set is the C0 control percent-encode set and U+0020 SPACE, U+0022 ("), U+003C (<), U+003E (>), and U+0060 (`).
pub const FRAGMENT_PERCENT_ENCODE_SET: &AsciiSet =
    &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
