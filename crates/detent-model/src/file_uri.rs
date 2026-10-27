//! The path a `file://` address names, as the file chooser portal answers with one.

/// The path a `file://` address names, with its `%XX` escapes read; `None` for any other address.
pub fn path_of_file_uri(uri: &str) -> Option<String> {
    let escaped = uri.strip_prefix("file://")?;
    let bytes = escaped.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let hex = |offset: usize| {
            bytes
                .get(at + offset)
                .and_then(|b| char::from(*b).to_digit(16))
        };
        match (bytes[at], hex(1), hex(2)) {
            (b'%', Some(high), Some(low)) => {
                out.push(u8::try_from(high * 16 + low).ok()?);
                at += 3;
            }
            (byte, _, _) => {
                out.push(byte);
                at += 1;
            }
        }
    }
    String::from_utf8(out)
        .ok()
        .filter(|path| path.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::path_of_file_uri;

    #[test]
    fn a_file_address_is_read_as_the_path_it_names_and_nothing_else_is() {
        const CASES: &[(&str, Option<&str>)] = &[
            ("file:///home/p/Strata", Some("/home/p/Strata")),
            (
                "file:///mnt/data/my%20models/%C3%A9",
                Some("/mnt/data/my models/\u{e9}"),
            ),
            ("https://example.org/x", None),
            ("file://host/x", None),
            ("file:///100%", Some("/100%")),
        ];
        for (uri, want) in CASES {
            assert_eq!(path_of_file_uri(uri).as_deref(), *want, "{uri}");
        }
    }
}
