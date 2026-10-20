//! A folder or a picture chosen by the person in the desktop's own chooser (the file chooser
//! portal), never typed: the one way a Strata they already have is pointed at, and the one way a
//! wallpaper is added.

use crate::live::{BusSource, connect};
use std::collections::HashMap;
use zbus::export::futures_core::Stream;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const CHOOSER: &str = "org.freedesktop.portal.FileChooser";
const REQUEST: &str = "org.freedesktop.portal.Request";

/// What the person is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Want {
    /// A folder.
    Folder,
    /// A picture file.
    Picture,
}

impl Want {
    /// Names the request, so its answer can be listened for before it is asked.
    fn token(self) -> &'static str {
        match self {
            Want::Folder => "detent_engine_folder",
            Want::Picture => "detent_wallpaper_picture",
        }
    }

    /// What the chooser's window is called.
    fn title(self) -> &'static str {
        match self {
            Want::Folder => "Choose a Folder",
            Want::Picture => "Add Picture",
        }
    }

    /// The chooser's options for it.
    fn options(self) -> HashMap<&'static str, Value<'static>> {
        let mut options = HashMap::from([("handle_token", Value::from(self.token()))]);
        if self == Want::Folder {
            options.insert("directory", Value::from(true));
        }
        options
    }
}

/// Asks the person for a folder or a picture, and waits for the answer; `None` when the chooser
/// is closed, there is none, or what was chosen is not a file on this computer.
pub(crate) async fn choose(bus: &BusSource, want: Want) -> Option<String> {
    let connection = connect(bus).await.ok()?;
    // The path the portal will give the request, from the name of this connection and the token.
    let sender = connection
        .unique_name()?
        .as_str()
        .trim_start_matches(':')
        .replace('.', "_");
    let request = zbus::Proxy::new(
        &connection,
        PORTAL,
        format!("{PORTAL_PATH}/request/{sender}/{}", want.token()),
        REQUEST,
    )
    .await
    .ok()?;
    let mut answers = request.receive_signal("Response").await.ok()?;
    let chooser = zbus::Proxy::new(&connection, PORTAL, PORTAL_PATH, CHOOSER)
        .await
        .ok()?;
    let _handle: OwnedObjectPath = chooser
        .call("OpenFile", &("", want.title(), want.options()))
        .await
        .ok()?;
    let answer = std::future::poll_fn(|cx| std::pin::Pin::new(&mut answers).poll_next(cx)).await?;
    let (code, results): (u32, HashMap<String, OwnedValue>) = answer.body().deserialize().ok()?;
    if code != 0 {
        return None;
    }
    let uris = Vec::<String>::try_from(results.get("uris")?.try_clone().ok()?).ok()?;
    path_of(uris.first()?)
}

/// The path a `file://` address names, with its `%XX` escapes read; `None` for any other address.
fn path_of(uri: &str) -> Option<String> {
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
    use super::path_of;

    #[test]
    fn a_file_address_is_read_as_the_path_it_names_and_nothing_else_is() {
        assert_eq!(
            path_of("file:///home/p/Strata"),
            Some("/home/p/Strata".to_owned())
        );
        assert_eq!(
            path_of("file:///mnt/data/my%20models/%C3%A9"),
            Some("/mnt/data/my models/\u{e9}".to_owned())
        );
        assert_eq!(path_of("https://example.org/x"), None);
        assert_eq!(path_of("file://host/x"), None);
        assert_eq!(path_of("file:///100%"), Some("/100%".to_owned()));
    }
}
